# Audio — Calibration & Test Tracks

This directory hosts calibration audio for EyeCandy. **Audio files themselves are gitignored** (live on FalconX at `C:\Users\Alexander\eyecandy\audio\`, may be copyrighted).

## Tracks

| Track | File | Status | Use |
|---|---|---|---|
| **Be Svendsen — Getula (Original)** | _to be sourced_ | ⏸ Pending acquisition (Beatport $1.49 WAV preferred) | **Phase 1 calibration anchor** — 108 BPM, A Major, 10-min long-form melodic house. All pipeline smoothing/refractory thresholds derived from this track. |
| THDS — Luz | `thds - Luz.wav` (55 MB, present) | ✅ On FalconX | Secondary test track. Tale of Us. Useful for verifying pipeline generalises beyond the calibration track. |

## Calibration philosophy

Per v3.1 §I.4: a single track defines the audio pipeline's tunable surface. Smoothing rates, kick-detector refractory, ISO 226 weighting, onset thresholds — all derived from Getula's signature. Other tracks inherit the defaults. We don't want a permanently re-tuning audio frontend; we want a calibrated frontend that works well-enough on everything else.
