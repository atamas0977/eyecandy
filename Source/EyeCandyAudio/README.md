# EyeCandyAudio — Source

Rust audio crate (ported from Stratum's `src/audio.rs`) + C++ FFI wrapper.

## Layout

```
rust/    Cargo workspace. cdylib target → libeyecandy_audio.dll on Windows, .so on Linux.
cpp/     C++ FFI wrapper compiled by UE5 plugin's Build.cs. Headers + thin shim around the Rust ABI.
```

## Audio features exposed (mirrors Stratum)

- ISO 226 perceptual weighting (loudness contour)
- Dolph–Chebyshev windowed FFT (low side-lobes, accurate spectrum)
- Two-stage smoothing: fast (~30 Hz) + slow (~3 Hz)
- Kick detector with refractory gating (calibrated against Getula's sparse kick pattern)
- Onset envelope, chroma vector (12-note), hilbert position vector

## Output schema (what UE5 reads each tick)

```
struct AudioFeatures {
    f32 bass_slow;      // smoothed bass band (~30..200 Hz), τ ~250 ms
    f32 bass_fast;      // fast bass band
    f32 mid_slow;       // mid band (~250 Hz..2 kHz), slow
    f32 mid_fast;       // mid band, fast
    f32 treble_slow;    // treble (~2..16 kHz), slow
    f32 treble_fast;    // treble, fast
    f32 kick_envelope;  // refractory-gated kick burst (0..1, fast decay)
    f32 audio_energy;   // global RMS, slow integrator (~8 s τ)
    f32 chroma[12];     // 12-note chroma (normalised)
    f32 hilbert_pos[3]; // X/Y/Z position from per-band Hilbert phase
}
```

UE5 plugin reads this struct each game-thread tick and writes scalars to a Material Parameter Collection.

## Inheritance from Stratum

Source-of-truth: `~/.openclaw/workspace/projects/stratum/src/audio.rs` (frozen). Port carries the calibrated thresholds verbatim, then re-calibrates against Getula in Phase 1.
