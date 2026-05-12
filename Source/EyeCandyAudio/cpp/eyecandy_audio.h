// SPDX-License-Identifier: MIT
// EyeCandy Audio C ABI header. Mirrors src/lib.rs EcaFeatures struct exactly.
// Personal-only project. Generated 2026-05-12 from EyeCandyAudio v0.1.0.

#ifndef EYECANDY_AUDIO_H
#define EYECANDY_AUDIO_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

// Layout MUST mirror EcaFeatures in Rust crate src/lib.rs.
// All values in [-1, 1] except *_pos which are in [-1, 1]^3.
typedef struct EcaFeatures_t {
    float bass_fast;
    float bass_slow;
    float mid_fast;
    float mid_slow;
    float treble_fast;
    float treble_slow;

    float kick_envelope;   // refractory-gated kick burst
    float audio_energy;    // global RMS

    float onset_envelope;  // peak-decay onset
    float bpm_estimate;    // 0 if not locked

    float chroma[12];      // 12-note chroma

    float bass_pos[3];     // 3D positions from per-band spectral centroid
    float mid_pos[3];
    float treble_pos[3];

    float _pad[1];
} EcaFeatures;

// Initialise capture. Returns 0 on success, 1 if already initialised,
// 2 on audio device error.
int32_t eca_init(void);

// Snapshot features into caller-provided struct. Thread-safe.
// Returns 0 on success, 1 if not initialised, 2 if `out` is null.
int32_t eca_get_features(EcaFeatures* out);

// Shutdown capture, join thread, drop resources. Returns 0.
int32_t eca_shutdown(void);

// Version string (static, null-terminated). Useful for plugin log.
const char* eca_version(void);

#ifdef __cplusplus
} // extern "C"
#endif

#endif // EYECANDY_AUDIO_H
