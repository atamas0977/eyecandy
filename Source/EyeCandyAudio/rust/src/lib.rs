//! EyeCandyAudio — Rust audio analyser exposed via C ABI for UE5.
//!
//! Architecture:
//! - `core` module: Stratum's audio pipeline verbatim (AudioCapture, FFT, ISO 226,
//!   2-stage smoothing, kick detector, chroma, hilbert positions).
//! - This file: C ABI shim. UE5 plugin loads `eyecandy_audio.dll` and calls
//!   `eca_init` / `eca_get_features` / `eca_shutdown` from C++.
//!
//! Threading model:
//! - `eca_init` creates an `AudioCapture` that spawns its own cpal capture thread.
//! - The cpal thread fills the internal smoothed-features state.
//! - `eca_get_features` is called from the UE5 game thread each tick; reads a snapshot.
//! - `eca_shutdown` joins the capture thread cleanly.
//!
//! Personal-only build. Not for redistribution.

#![allow(clippy::missing_safety_doc)] // C ABI is unsafe by definition; safety contract in comments.

pub mod core;

use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use crate::core::{AudioCapture, AudioFeatures};

/// Singleton: only one AudioCapture per process. UE5 plugin shouldn't init twice.
static CAPTURE: OnceCell<Mutex<Option<AudioCapture>>> = OnceCell::new();

fn capture_slot() -> &'static Mutex<Option<AudioCapture>> {
    CAPTURE.get_or_init(|| Mutex::new(None))
}

// ---------- C ABI ----------

/// FFI-stable mirror of `AudioFeatures` for C++ consumption. Layout MUST match
/// `EyeCandyAudio.h` on the C++ side. Padding fields keep alignment explicit.
///
/// Order: smoothed bands (fast/slow), kick, energy, chroma[12], hilbert positions.
/// Bins are exposed as a separate API (`eca_get_bins`) to keep this struct small.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct EcaFeatures {
    pub bass_fast: f32,
    pub bass_slow: f32,
    pub mid_fast: f32,
    pub mid_slow: f32,
    pub treble_fast: f32,
    pub treble_slow: f32,

    pub kick_envelope: f32,
    pub audio_energy: f32, // global RMS, slow integrator

    pub onset_envelope: f32, // peak-decay onset detector
    pub bpm_estimate: f32,   // tempo guess (Hz / spectral autocorrelation), 0 if not yet locked

    pub chroma: [f32; 12], // 12-note chroma (normalised)

    pub bass_pos: [f32; 3], // 3D positions from per-band spectral centroid
    pub mid_pos: [f32; 3],
    pub treble_pos: [f32; 3],

    // 16-byte alignment padding for predictable struct size across compilers
    pub _pad: [f32; 1],
}

impl Default for EcaFeatures {
    fn default() -> Self {
        EcaFeatures {
            bass_fast: 0.0, bass_slow: 0.0,
            mid_fast: 0.0, mid_slow: 0.0,
            treble_fast: 0.0, treble_slow: 0.0,
            kick_envelope: 0.0, audio_energy: 0.0,
            onset_envelope: 0.0, bpm_estimate: 0.0,
            chroma: [0.0; 12],
            bass_pos: [0.0; 3], mid_pos: [0.0; 3], treble_pos: [0.0; 3],
            _pad: [0.0],
        }
    }
}

/// Convert Stratum's internal `AudioFeatures` → FFI struct.
///
/// Note: Stratum's struct currently exposes `bass`, `mid`, `treble` (fast) and
/// `bass_slow`, `mid_slow`, `treble_slow` (slow). Map accordingly. The Stratum
/// struct does not yet have `onset_envelope` / `bpm_estimate` / `chroma` —
/// those are Phase 1 extensions. For now they're zeroed; the v3.1 audio crate
/// port adds them as part of the calibration work against Getula.
fn snapshot(s: &AudioFeatures) -> EcaFeatures {
    let mut out = EcaFeatures::default();
    out.bass_fast = s.bass;
    out.mid_fast = s.mid;
    out.treble_fast = s.treble;
    out.bass_slow = s.bass_slow;
    out.mid_slow = s.mid_slow;
    out.treble_slow = s.treble_slow;
    out.kick_envelope = s.kick;
    out.audio_energy = s.loudness;
    out.onset_envelope = s.onset;
    // bpm_estimate: not yet computed by Stratum core; left at 0.0. Phase 1 extension.
    out.bass_pos = s.bass_pos;
    out.mid_pos = s.mid_pos;
    out.treble_pos = s.treble_pos;
    // chroma: Phase 1 extension (not in Stratum core)
    out
}

/// Initialise the audio capture. Returns 0 on success, non-zero on error.
/// Safe to call once. Calling twice without `eca_shutdown` returns 1.
#[no_mangle]
pub unsafe extern "C" fn eca_init() -> i32 {
    let slot = capture_slot();
    let mut guard = slot.lock();
    if guard.is_some() {
        return 1; // already initialised
    }
    match AudioCapture::start() {
        Ok(cap) => {
            *guard = Some(cap);
            0
        }
        Err(e) => {
            eprintln!("[eyecandy_audio] eca_init failed: {e:#}");
            2
        }
    }
}

/// Snapshot the current audio features into the caller-provided struct.
/// Safe to call from any thread, including the UE5 game thread.
///
/// Returns 0 on success. Returns 1 if capture not initialised.
/// `out` must be non-null and pointing to writable `EcaFeatures`.
#[no_mangle]
pub unsafe extern "C" fn eca_get_features(out: *mut EcaFeatures) -> i32 {
    if out.is_null() {
        return 2;
    }
    let slot = capture_slot();
    let guard = slot.lock();
    let Some(cap) = guard.as_ref() else {
        return 1;
    };
    let snap = snapshot(&cap.snapshot_features());
    *out = snap;
    0
}

/// Shutdown the capture, join the cpal thread, drop resources.
/// Returns 0 always.
#[no_mangle]
pub unsafe extern "C" fn eca_shutdown() -> i32 {
    let slot = capture_slot();
    let mut guard = slot.lock();
    *guard = None; // Drop runs cpal stream teardown
    0
}

/// Library version string (null-terminated, static). Useful for UE5 to log
/// what's loaded.
#[no_mangle]
pub unsafe extern "C" fn eca_version() -> *const i8 {
    static V: &[u8] = b"eyecandy_audio 0.1.0\0";
    V.as_ptr() as *const i8
}

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_string_is_valid_cstr() {
        unsafe {
            let p = eca_version();
            assert!(!p.is_null());
            // Read until NUL
            let mut len = 0;
            while *p.add(len) != 0 { len += 1; assert!(len < 64); }
            assert!(len > 0);
        }
    }
}
