# Audio — Calibration & Test Tracks

This directory hosts calibration audio for EyeCandy. **Audio files themselves are gitignored** (live on FalconX at `C:\Users\Alexander\eyecandy\audio\`, may be copyrighted).

## Tracks

| Track | File | Status | Use |
|---|---|---|---|
| **Be Svendsen — Getula (Original)** | `Be Svendsen - Getula (Original).mp3` (23.1 MB, 320 kbps MP3, 44.1 kHz stereo) | ✅ On FalconX (`C:\Users\Alexander\eyecandy\audio\`) | **Phase 1 calibration anchor** — 108 BPM, A Major, 10-min long-form melodic house. All pipeline smoothing/refractory thresholds derived from this track. Sourced from AT's Drive folder 2026-05-12 15:03. |
| THDS — Luz | `thds - Luz.wav` (52.9 MB, present) | ✅ On FalconX | Secondary test track. Tale of Us. Useful for verifying pipeline generalises beyond the calibration track. |

## Calibration philosophy

Per v3.1 §I.4: a single track defines the audio pipeline's tunable surface. Smoothing rates, kick-detector refractory, ISO 226 weighting, onset thresholds — all derived from Getula's signature. Other tracks inherit the defaults. We don't want a permanently re-tuning audio frontend; we want a calibrated frontend that works well-enough on everything else.
