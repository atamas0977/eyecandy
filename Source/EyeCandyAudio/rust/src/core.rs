//! Audio capture and feature extraction.
//!
//! Captures system output (loopback) and runs an FFT each frame to produce the
//! `AudioFeatures` struct that the renderer consumes as a uniform buffer.
//!
//! v0.5.2 — audio pipeline overhaul (ports from fractal_sugar + MuTate):
//!  - Two-stage frame-rate-independent exponential smoothing (fast/slow envelopes)
//!    with intensity-modulated fast rate (`0.36 * sqrt(intensity)`).
//!  - ISO 226:2023 equal-loudness weighting per FFT bin (bass +20dB vs 1 kHz).
//!  - Adaptive kick detector: 16-frame moving average + 0.8s refractory.
//!    Distinct `kick_envelope` decays exponentially over ~150ms.
//!  - bass_pos / mid_pos / treble_pos: 3D positions driven by per-band spectral
//!    centroid; consumed by orbit-trap fractal scenes.
//!
//! Platform notes:
//! - **Windows**: WASAPI loopback. cpal's default output device with the loopback
//!   feature gives us system audio (everything the user hears).
//! - **Linux**: PipeWire monitor source via cpal's ALSA backend.
//! - **macOS**: requires a virtual loopback driver (BlackHole, Soundflower).
//!
//! Configuration: stereo input @ 48 kHz, 1024-sample FFT, 3 bands.

use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use parking_lot::Mutex;
use rustfft::{num_complex::Complex32, FftPlanner};
use std::sync::Arc;

/// Number of FFT bins we hand to the GPU as a uniform.
pub const NUM_BINS: usize = 64;

/// Raw audio features pushed to the GPU each frame.
///
/// `repr(C)` + `Pod` + `Zeroable` so we can `bytemuck::cast_slice` it straight
/// into a wgpu uniform buffer. Layout MUST match the WGSL `Audio` struct exactly.
///
/// Layout notes (WGSL std140-ish):
///  - `array<vec4<f32>, 16>` for bins (16 × 16 = 256 bytes, aligned 16).
///  - Each `vec3<f32>` field is followed by an explicit `f32` pad so the next
///    field starts on a 16-byte boundary.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AudioFeatures {
    /// 64 mel-ish frequency bins, normalized 0..1 (packed 4 per vec4).
    pub bins: [f32; NUM_BINS],
    /// fast-smoothed bass amplitude 0..1 (intensity-modulated rate)
    pub bass: f32,
    /// fast-smoothed mid amplitude 0..1
    pub mid: f32,
    /// fast-smoothed treble amplitude 0..1
    pub treble: f32,
    /// broadband loudness 0..1
    pub loudness: f32,
    /// time in seconds since startup
    pub time: f32,
    /// onset envelope 0..1 (decays smoothly; fires on any transient)
    pub onset: f32,
    /// aspect ratio of render target (set by renderer)
    pub aspect: f32,
    /// per-scene variant selector (0..N)
    pub variant: f32,

    // --- v0.5.2 additions ---

    /// slow-smoothed bass (constant 0.15/s rate — for camera drift)
    pub bass_slow: f32,
    /// slow-smoothed mid
    pub mid_slow: f32,
    /// slow-smoothed treble
    pub treble_slow: f32,
    /// kick envelope 0..1 (adaptive kick detector, ~150ms decay).
    /// Distinct from `onset`: kicks are bass-only with 0.8s refractory.
    pub kick: f32,

    /// fast-smoothed 3D position for bass, driven by spectral centroid in [-1, 1]^3.
    /// vec3 + pad to 16 bytes.
    pub bass_pos: [f32; 3],
    pub _pad_bass_pos: f32,
    pub mid_pos: [f32; 3],
    pub _pad_mid_pos: f32,
    pub treble_pos: [f32; 3],
    pub _pad_treble_pos: f32,
}

/// Tempo / beat tracking state. CPU-side only (combinator reads this for BPM-synced merges).
#[derive(Copy, Clone, Debug, Default)]
pub struct BeatState {
    pub bpm: f32,
    pub last_onset_time: f32,
    pub beats: u32,
}

impl Default for AudioFeatures {
    fn default() -> Self {
        Self {
            bins: [0.0; NUM_BINS],
            bass: 0.0,
            mid: 0.0,
            treble: 0.0,
            loudness: 0.0,
            time: 0.0,
            onset: 0.0,
            aspect: 16.0 / 9.0,
            variant: 0.0,
            bass_slow: 0.0,
            mid_slow: 0.0,
            treble_slow: 0.0,
            kick: 0.0,
            bass_pos: [0.0; 3],
            _pad_bass_pos: 0.0,
            mid_pos: [0.0; 3],
            _pad_mid_pos: 0.0,
            treble_pos: [0.0; 3],
            _pad_treble_pos: 0.0,
        }
    }
}

/// FFT-side state shared between the audio thread (writing) and render thread (reading).
struct SharedState {
    features: AudioFeatures,
    beat: BeatState,
    samples: Vec<f32>,
    write_pos: usize,
    start: std::time::Instant,
    /// Last process_block instant — used to compute audio-thread dt for smoothing.
    last_process_time: std::time::Instant,
    /// Smoothed bass for onset detection (slow-following baseline).
    bass_baseline: f32,
    last_bass: f32,
    onset_env: f32,
    auto_gain: f32,
    onset_intervals: Vec<f32>,

    // --- v0.5.2: kick detector state ---
    /// 16-frame history of raw bass magnitude (post auto-gain, pre-smoothing).
    kick_history: [f32; 16],
    kick_idx: usize,
    last_kick: std::time::Instant,
    kick_env: f32,
}

pub struct AudioCapture {
    _stream: cpal::Stream,
    state: Arc<Mutex<SharedState>>,
    sample_rate: u32,
    device_name: String,
}

impl AudioCapture {
    pub fn start() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow!("no default output device"))?;

        let device_name = device.name().unwrap_or_else(|_| "<unnamed>".into());
        eprintln!("[eca] audio device: {device_name}");

        let config = device
            .default_output_config()
            .context("getting default output config")?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;

        eprintln!(
            "[eca] audio: {sample_rate} Hz x {channels} ch, format {:?}",
            config.sample_format()
        );

        let now = std::time::Instant::now();
        let state = Arc::new(Mutex::new(SharedState {
            features: AudioFeatures::default(),
            beat: BeatState::default(),
            samples: vec![0.0; 4096],
            write_pos: 0,
            start: now,
            last_process_time: now,
            bass_baseline: 0.05,
            last_bass: 0.0,
            onset_env: 0.0,
            auto_gain: 1.0,
            onset_intervals: Vec::with_capacity(32),
            kick_history: [0.0; 16],
            kick_idx: 0,
            last_kick: now,
            kick_env: 0.0,
        }));

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(1024);

        let cb_state = state.clone();
        let cb_sr = sample_rate;
        let cb_ch = channels;
        let cb_fft = fft;

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config.into(), cb_state, cb_sr, cb_ch, cb_fft)?,
            cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config.into(), cb_state, cb_sr, cb_ch, cb_fft)?,
            cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config.into(), cb_state, cb_sr, cb_ch, cb_fft)?,
            other => return Err(anyhow!("unsupported sample format: {other:?}")),
        };

        stream.play().context("starting audio stream")?;

        Ok(Self {
            _stream: stream,
            state,
            sample_rate,
            device_name,
        })
    }

    pub fn snapshot_features(&self) -> AudioFeatures {
        self.state.lock().features
    }

    pub fn snapshot_beat(&self) -> BeatState {
        self.state.lock().beat
    }

    pub fn set_aspect(&self, aspect: f32) {
        let mut s = self.state.lock();
        s.features.aspect = aspect;
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    state: Arc<Mutex<SharedState>>,
    sample_rate: u32,
    channels: usize,
    fft: Arc<dyn rustfft::Fft<f32>>,
) -> Result<cpal::Stream>
where
    T: cpal::SizedSample + Send + 'static,
    f32: cpal::FromSample<T>,
{
    let err_cb = |e| eprintln!("[eca] audio stream error: {e:?}");

    let stream = device
        .build_input_stream(
            config,
            move |data: &[T], _info: &cpal::InputCallbackInfo| {
                process_block::<T>(data, &state, sample_rate, channels, &fft);
            },
            err_cb,
            None,
        )
        .context("build_input_stream")?;
    Ok(stream)
}

// ─────────────────────────────────────────────────────────────────────────────
// ISO 226:2023 equal-loudness weighting (perceptual gain per frequency).
//
// Source: ISO 226:2023 Table 1 (70 phons reference). Public-domain standard.
// We tabulate SPL needed at each frequency to match the loudness of 70 dB SPL
// at 1 kHz, then return (ref_spl_1khz - target_spl_at_f) in dB. Adding this
// to the measured bin magnitude (in dB) converts it to a perceptually-flat
// value: bass at 50 Hz gets +20 dB, 1 kHz gets 0 dB, etc.
//
// Ported clean-room from the standard (the table is verbatim ISO data, which
// is not copyrightable; the implementation is original).
// ─────────────────────────────────────────────────────────────────────────────
const ISO226_FREQS: &[f32] = &[
    20.0, 25.0, 31.5, 40.0, 50.0, 63.0, 80.0, 100.0, 125.0, 160.0, 200.0,
    250.0, 315.0, 400.0, 500.0, 630.0, 800.0, 1000.0, 1250.0, 1600.0,
    2000.0, 2500.0, 3150.0, 4000.0, 5000.0, 6300.0, 8000.0, 10000.0, 12500.0,
];
const ISO226_SPL70: &[f32] = &[
    100.83, 93.94, 87.34, 80.50, 74.17, 68.61, 63.40, 58.96, 55.31, 52.06, 49.20,
    46.71, 44.50, 42.49, 40.94, 39.89, 39.42, 40.00, 41.71, 42.75, 41.96, 39.92,
    36.97, 33.41, 32.10, 33.41, 39.42, 48.39, 54.81,
];

/// Returns the dB gain to add to a magnitude measured at `freq_hz` so that
/// it represents the same perceived loudness as a 1 kHz reference. Frequencies
/// below 20 Hz or above 12.5 kHz are clamped to the table endpoints.
fn iso226_gain_db(freq_hz: f32) -> f32 {
    // SPL70[f] is the SPL needed at f to match 70-phon loudness at 1 kHz.
    // Higher SPL needed = ear is less sensitive = boost more in our signal.
    // So gain = SPL70[f] - SPL70[1 kHz]. (Bass needs +34 dB; 4 kHz gets -6 dB.)
    let ref_spl = lerp_iso226(1000.0);
    let target_spl = lerp_iso226(freq_hz);
    target_spl - ref_spl
}

fn lerp_iso226(f: f32) -> f32 {
    if f <= ISO226_FREQS[0] {
        return ISO226_SPL70[0];
    }
    let n = ISO226_FREQS.len();
    if f >= ISO226_FREQS[n - 1] {
        return ISO226_SPL70[n - 1];
    }
    // Find bracketing index. Table is small (29 entries) — linear scan is fine.
    for i in 1..n {
        if ISO226_FREQS[i] >= f {
            let f0 = ISO226_FREQS[i - 1];
            let f1 = ISO226_FREQS[i];
            let v0 = ISO226_SPL70[i - 1];
            let v1 = ISO226_SPL70[i];
            // Linear interp in log-frequency space — bins are log-spaced perceptually.
            let t = (f.ln() - f0.ln()) / (f1.ln() - f0.ln());
            return v0 + (v1 - v0) * t;
        }
    }
    ISO226_SPL70[n - 1]
}

/// Per-bin linear-domain perceptual multiplier. Multiply magnitudes by this.
fn iso226_gain_linear(freq_hz: f32) -> f32 {
    10.0f32.powf(iso226_gain_db(freq_hz) / 20.0)
}

// ─────────────────────────────────────────────────────────────────────────────
// Frame-rate-independent exponential smoothing (fractal_sugar technique).
//
//   *a += (target - *a) * (1 - exp(-rate * dt))
//
// Stable for any dt. Rate is in units of "per second."
// ─────────────────────────────────────────────────────────────────────────────
#[inline]
fn lerp_exp(a: &mut f32, target: f32, rate_per_sec: f32, dt: f32) {
    let k = 1.0 - (-rate_per_sec * dt).exp();
    *a += (target - *a) * k;
}

#[inline]
fn lerp_exp_vec3(a: &mut [f32; 3], target: &[f32; 3], rate_per_sec: f32, dt: f32) {
    let k = 1.0 - (-rate_per_sec * dt).exp();
    for i in 0..3 {
        a[i] += (target[i] - a[i]) * k;
    }
}

/// Map a normalized frequency `t ∈ [0,1]` to a 3D position in `[-1, 1]^3`.
/// Cheap surrogate for the Hilbert curve: trig-based "knotted" path that
/// gives non-axis-aligned spread. Nearby frequencies → nearby positions.
fn freq_to_pos(t: f32) -> [f32; 3] {
    // Three coprime frequency multipliers + phase offsets — produces a
    // pseudo-knotted curve through the cube without trivial axis-alignment.
    let u = t.clamp(0.0, 1.0);
    let x = (u * 1.7 * std::f32::consts::TAU + 0.1).sin();
    let y = (u * 2.3 * std::f32::consts::TAU + 1.3).sin();
    let z = (u * 3.1 * std::f32::consts::TAU + 2.7).sin();
    [x, y, z]
}

fn process_block<T>(
    block: &[T],
    state: &Arc<Mutex<SharedState>>,
    sample_rate: u32,
    channels: usize,
    fft: &Arc<dyn rustfft::Fft<f32>>,
)
where
    T: cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let mut s = state.lock();

    // Audio-thread dt (used for frame-rate-independent smoothing).
    let now = std::time::Instant::now();
    let dt = (now - s.last_process_time).as_secs_f32().clamp(0.001, 0.1);
    s.last_process_time = now;

    // Mix to mono and append to ring buffer.
    let frames = block.len() / channels.max(1);
    for f in 0..frames {
        let mut sum = 0.0f32;
        for c in 0..channels {
            sum += f32::from_sample(block[f * channels + c]);
        }
        let mono = sum / channels.max(1) as f32;
        let pos = s.write_pos;
        s.samples[pos] = mono;
        s.write_pos = (s.write_pos + 1) % s.samples.len();
    }

    // Pull the most recent 1024 samples for FFT.
    const N: usize = 1024;
    let mut buf = [Complex32::new(0.0, 0.0); N];
    let len = s.samples.len();
    let start = (s.write_pos + len - N) % len;
    for i in 0..N {
        let idx = (start + i) % len;
        // Hann window — soften FFT spectral leakage
        let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (N as f32 - 1.0)).cos());
        buf[i] = Complex32::new(s.samples[idx] * w, 0.0);
    }

    fft.process(&mut buf);

    // Magnitudes. First half is meaningful (Nyquist).
    // ISO 226 perceptual weighting per FFT bin (Port 2).
    // `freq_per_bin = sample_rate / N`. Apply before any aggregation.
    let freq_per_bin = sample_rate as f32 / N as f32;
    let mut mags = [0.0f32; N / 2];
    for i in 0..N / 2 {
        let raw = buf[i].norm();
        let bin_freq = (i as f32 + 0.5) * freq_per_bin;
        // ISO226 gain is in dB; cap to ~+25 dB to avoid blow-up on the lowest bins
        // (the SPL table starts at 20 Hz; sub-bass below that gets clamped).
        let gain_db = iso226_gain_db(bin_freq).clamp(-10.0, 25.0);
        let gain_linear = 10.0f32.powf(gain_db / 20.0);
        mags[i] = raw * gain_linear;
    }

    // Normalize: log-mapping + soft clip. After ISO 226 weighting the magnitudes
    // grew (bass especially), so we slightly back off the scale so tanh in the
    // band stage doesn't permanently peg high.
    let _ = iso226_gain_linear; // keep helper alive for callers/tests
    let mut norm = [0.0f32; N / 2];
    for i in 0..N / 2 {
        // After ISO226: bass bins can be ~6-10x larger. Scale 0.025 keeps peaks
        // in a similar range to pre-v0.5.2; tanh in band stage absorbs the rest.
        let v = (mags[i] * 0.025).ln_1p() / 4.0;
        norm[i] = v.clamp(0.0, 1.0);
    }

    // Downsample to NUM_BINS log-spaced bins.
    let mut bins = [0.0f32; NUM_BINS];
    let nyquist = sample_rate as f32 / 2.0;
    for b in 0..NUM_BINS {
        let f0 = 30.0 * (nyquist / 30.0).powf(b as f32 / NUM_BINS as f32);
        let f1 = 30.0 * (nyquist / 30.0).powf((b as f32 + 1.0) / NUM_BINS as f32);
        let i0 = ((f0 / nyquist) * (N as f32 / 2.0)) as usize;
        let i1 = ((f1 / nyquist) * (N as f32 / 2.0)) as usize;
        let i1 = i1.min(N / 2);
        let i0 = i0.min(i1);
        let count = (i1 - i0).max(1);
        let sum: f32 = (i0..i1).map(|i| norm[i]).sum();
        bins[b] = sum / count as f32;
    }

    let bin_of = |hz: f32| -> usize {
        ((hz * N as f32 / sample_rate as f32) as usize).min(N / 2)
    };
    let bass_raw = avg_range(&norm, bin_of(30.0), bin_of(200.0));
    let mid_raw = avg_range(&norm, bin_of(200.0), bin_of(2000.0));
    let treble_raw = avg_range(&norm, bin_of(2000.0), bin_of(8000.0));
    let loud_raw = (bass_raw + mid_raw + treble_raw) / 3.0;

    // ---- Spectral centroid per band (drives 3D positions) ----
    // Within each band's bin range, compute the magnitude-weighted center.
    // Map t ∈ [0,1] → 3D via freq_to_pos.
    let centroid = |lo: usize, hi: usize| -> f32 {
        let mut num = 0.0;
        let mut den = 0.0;
        for i in lo..hi {
            num += (i as f32) * norm[i];
            den += norm[i];
        }
        if den < 1e-6 || hi <= lo {
            return 0.5;
        }
        let center_bin = num / den;
        ((center_bin - lo as f32) / (hi - lo) as f32).clamp(0.0, 1.0)
    };
    let bass_c = centroid(bin_of(30.0), bin_of(200.0));
    let mid_c = centroid(bin_of(200.0), bin_of(2000.0));
    let treble_c = centroid(bin_of(2000.0), bin_of(8000.0));
    let bass_pos_target = freq_to_pos(bass_c.powf(0.84));
    let mid_pos_target = freq_to_pos(mid_c.powf(0.75));
    let treble_pos_target = freq_to_pos(treble_c.powf(0.445));

    // ---- Auto-gain ----
    let target_loud: f32 = 0.3;
    if loud_raw > 0.005 {
        let desired_gain = (target_loud / loud_raw).clamp(0.5, 2.5);
        let alpha = if desired_gain > s.auto_gain { 0.003 } else { 0.012 };
        s.auto_gain += (desired_gain - s.auto_gain) * alpha;
    }
    let g = s.auto_gain;

    let soft = |v: f32| -> f32 { (v * 2.0).tanh().clamp(0.0, 1.0) };
    let bass_target = soft(bass_raw * g);
    let mid_target = soft(mid_raw * g * 1.4);
    // ISO226 already boosts treble bins, so the prior 3.5× hard-trim is too hot.
    let treble_target = soft(treble_raw * g * 2.0);
    let loud_target = soft(loud_raw * g);

    // ---- Port 1: Two-stage frame-rate-independent exponential smoothing ----
    //
    //   reactive (this frame's target) → fast → slow
    //
    // BASE_SPEED converts fractal_sugar's "per-frame at ~60Hz" rates to "per-second."
    // fast_rate is intensity-modulated: `0.36 * sqrt(intensity) * BASE_SPEED`.
    // slow_rate is constant `0.15 * BASE_SPEED`.
    //
    // Why two stages:
    //   - `*_fast` is used by per-pixel reactivity (orbit traps, attractors).
    //     Snaps on transients, drifts on calm sections.
    //   - `*_slow` is used by camera/world drift. Slow rhythmic sway, never jittery.
    const BASE_SPEED: f32 = 4.0;
    let intensity = loud_target.clamp(0.0, 1.0);
    let fast_rate = 0.36 * (0.8 * intensity.sqrt()).min(1.0).max(0.1) * BASE_SPEED;
    let slow_rate = 0.15 * BASE_SPEED;

    // Stage 1: target → fast
    let mut f = s.features;
    lerp_exp(&mut f.bass, bass_target, fast_rate, dt);
    lerp_exp(&mut f.mid, mid_target, fast_rate, dt);
    lerp_exp(&mut f.treble, treble_target, fast_rate, dt);
    lerp_exp(&mut f.loudness, loud_target, fast_rate, dt);
    lerp_exp_vec3(&mut f.bass_pos, &bass_pos_target, fast_rate, dt);
    lerp_exp_vec3(&mut f.mid_pos, &mid_pos_target, fast_rate, dt);
    lerp_exp_vec3(&mut f.treble_pos, &treble_pos_target, fast_rate, dt);

    // Stage 2: fast → slow
    lerp_exp(&mut f.bass_slow, f.bass, slow_rate, dt);
    lerp_exp(&mut f.mid_slow, f.mid, slow_rate, dt);
    lerp_exp(&mut f.treble_slow, f.treble, slow_rate, dt);

    // ---- Onset detection (general transient — kept for back-compat) ----
    let baseline_alpha: f32 = 0.04;
    s.bass_baseline = s.bass_baseline + (f.bass - s.bass_baseline) * baseline_alpha;
    let rising = f.bass - s.last_bass;
    s.last_bass = f.bass;
    let triggered = f.bass > s.bass_baseline * 1.35
        && rising > 0.05
        && f.bass > 0.20
        && s.onset_env < 0.4;
    if triggered {
        s.onset_env = 1.0;
        let now_sec = s.start.elapsed().as_secs_f32();
        let last = s.beat.last_onset_time;
        if last > 0.0 {
            let dt_onset = now_sec - last;
            if dt_onset > 0.25 && dt_onset < 1.5 {
                s.onset_intervals.push(dt_onset);
                if s.onset_intervals.len() > 16 {
                    s.onset_intervals.remove(0);
                }
                let mut sorted = s.onset_intervals.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let median = sorted[sorted.len() / 2];
                s.beat.bpm = 60.0 / median.max(0.001);
            }
        }
        s.beat.last_onset_time = now_sec;
        s.beat.beats = s.beat.beats.saturating_add(1);
    } else {
        s.onset_env *= 0.88;
    }
    let onset = s.onset_env;

    // ---- Port 3: Adaptive kick detector ----
    //
    // History over the last 16 bass magnitudes; trigger when:
    //   (mag > 4 × scale-of-typical-bass) OR (mag * elapsed > 8 × scale)
    //   AND elapsed > 0.8s (refractory)
    //   AND mag > 1.25 × scale
    //   AND mag > 3 × moving-average
    //
    // The "scale" mismatch: fractal_sugar's `bass.mag` lived in raw magnitude
    // space; Stratum's `bass_raw` is the post-norm 0..1 value. We rescale
    // thresholds by 0.25 to keep the relative dynamics.
    let bass_mag = (bass_raw * g).clamp(0.0, 5.0); // pre-tanh, post-gain — preserves dynamics
    let kick_idx = s.kick_idx;
    s.kick_history[kick_idx] = bass_mag;
    s.kick_idx = (kick_idx + 1) % s.kick_history.len();
    let kick_avg: f32 =
        s.kick_history.iter().copied().sum::<f32>() / s.kick_history.len() as f32;
    let kick_elapsed = (now - s.last_kick).as_secs_f32();

    // Thresholds (scaled from fractal_sugar's raw-magnitude domain by 0.25):
    //   raw   → stratum
    //   4.0   → 1.0
    //   8.0   → 2.0  (mag * elapsed)
    //   1.25  → 0.31
    //   3.0×  → 3.0× (ratio is unitless, unchanged)
    let kick_triggered = (bass_mag > 1.0 || bass_mag * kick_elapsed > 2.0)
        && kick_elapsed > 0.8
        && bass_mag > 0.31
        && bass_mag > 3.0 * kick_avg;
    if kick_triggered {
        s.last_kick = now;
        s.kick_env = 1.0;
    } else {
        // Exponential decay over ~150ms: e^(-dt/tau), tau ≈ 0.05s -> half-life ~35ms.
        // Use tau=0.075 for a ~150ms perceptual decay envelope.
        let tau = 0.075;
        s.kick_env *= (-dt / tau).exp();
    }
    f.kick = s.kick_env;

    let time = s.start.elapsed().as_secs_f32();
    let aspect = s.features.aspect;
    let variant = s.features.variant;

    f.bins = bins;
    f.onset = onset;
    f.time = time;
    f.aspect = aspect;
    f.variant = variant;
    s.features = f;
}

fn avg_range(slice: &[f32], a: usize, b: usize) -> f32 {
    if b <= a {
        return 0.0;
    }
    let len = (b - a) as f32;
    slice[a..b].iter().sum::<f32>() / len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso226_reference_is_zero_at_1khz() {
        // By construction, ref_spl - target_spl == 0 at the reference frequency.
        let g = iso226_gain_db(1000.0);
        assert!(g.abs() < 0.01, "expected ~0 dB at 1 kHz, got {g}");
    }

    #[test]
    fn iso226_boosts_bass() {
        // 50 Hz should get a substantial positive boost (~+20 dB).
        let g = iso226_gain_db(50.0);
        assert!(g > 15.0, "expected >15 dB boost at 50 Hz, got {g}");
    }

    #[test]
    fn iso226_lerp_handles_endpoints() {
        // Below table → table[0] value.
        let g = iso226_gain_db(10.0);
        assert!(g > 30.0); // 20 Hz is at ~100 SPL = ~60 dB above 1 kHz's ~40 SPL
    }
}
