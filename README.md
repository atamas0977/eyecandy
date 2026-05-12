# EyeCandy

Audio-reactive cinematic-scene renderer for FalconX (RTX 5090, triple 4K).
UE5 NvRTX 5.6 + path tracing + Rust audio pipeline.

**Personal-only. Never redistributed.**

## Status

Phase 0 — infrastructure stand-up. See `docs/STATUS.md` for live state.

## Plan

- v3 thesis: `dashboard/files/conversations/eyecandy-thesis-v3-2026-05-11.pdf`
- v3.1 addendum (Phase 1 = Bonsai, Phase 2 = City Sample): `dashboard/files/conversations/eyecandy-thesis-v3.1-addendum-2026-05-11.pdf`

## Layout (Quintus-side workspace)

This directory at `/home/atamas/.openclaw/workspace/projects/eyecandy/` is the **Quintus-side** mirror — code, design notes, audio crate source, plugin source. Heavy artifacts (engine, Bonsai, City Sample, derived data) live only on FalconX.

```
Source/EyeCandyAudio/rust/    — Rust audio crate (ported from Stratum)
Source/EyeCandyAudio/cpp/     — C++ FFI wrapper
Plugins/EyeCandyAudio/        — UE5 plugin (loads Rust cdylib over FFI)
Plugins/EyeCandyBindings/     — UE5 plugin (MPC scalar drivers, scene bindings)
Content/EyeCandy/             — UE5 content (MPCs, Blueprints, materials)
scenes/                       — scene packaging notes, asset manifests
audio/                        — calibration tracks (gitignored, hosted on FalconX)
scripts/                      — sync scripts, launch scripts, build scripts
docs/                         — design notes, status, decisions
```

## Layout (FalconX side)

`C:\Users\Alexander\eyecandy\` on FalconX (192.168.0.145).
```
assets/nvrtx-engine/                 — NvRTX 5.6 engine, source-built 2026-05-11/12
assets/bonsai-binary/                — NVIDIA Bonsai binary distribution (Phase 1 anchor)
zenparticles-ai/BonsaiDiorama/       — Bonsai project source (Phase 1 anchor, source variant)
audio/                               — Calibration tracks (thds - Luz.wav now, Getula incoming)
scenes/  src/  plugins/              — Synced from this Quintus directory
NVIDIA RTX Bonsai Diorama - Developer Guide.pdf
```

## Engine

- **NvRTX 5.6** source-built on FalconX, GUID `{6322DDB0-470D-9A69-C2AB-97A502DD2ED5}`
- All 5 helper binaries built (`ShaderCompileWorker`, `UnrealLightmass`, `InterchangeWorker`, `CrashReportClient`, `EpicWebHelper`)
- See `docs/build-notes.md` for the build-helpers gotchas

## Hardware

- **FalconX**: RTX 5090 32GB, driver 591.86, Windows 11 64-bit, VS 2022 Community
- **Quintus-Maximus**: ThinkStation P3 Tiny + iGPU (code-only; not for renderer iteration)

## Phase 1 anchor: Bonsai Diorama

- Phase 1 ships when audio bindings (`KeyLightIntensity`, `TimeOfDayPhase`, `FogDensity`, `GlassGlintEmissive`) drive Bonsai from the Stratum-derived Rust audio crate via MPC scalars.
- Phase 1 anchor track: Getula by Be Svendsen (108 BPM, A Major). Calibration target.
- Phase 1 stretch: triple-4K via DLSS-RR + MFG 3× sustained ≥60 FPS on FalconX Surround config.

## Phase 2 anchor: City Sample (Matrix Awakens)

- Lumen-first, Path-trace overlay-mode.
- Defer install until Phase 2.
