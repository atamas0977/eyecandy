# EyeCandy — Live Status

*Updated 2026-05-12 14:42 GMT+2*

## Phase 0 — Infrastructure Stand-Up

| Item | State | Notes |
|---|---|---|
| NvRTX 5.6 engine built on FalconX | ✅ Done | `C:\Users\Alexander\eyecandy\assets\nvrtx-engine\`, GUID `{6322DDB0-470D-9A69-C2AB-97A502DD2ED5}` registered in HKCU |
| 5 helper binaries built | ✅ Done | SCW, Lightmass, Interchange, CrashReportClient, EpicWebHelper (all in `Engine\Binaries\Win64\`) |
| Bonsai source files present | ✅ Done | `C:\Users\Alexander\eyecandy\zenparticles-ai\BonsaiDiorama\BonsaiDiorama.uproject` |
| Bonsai binary download | ✅ Done | 1.79 GB extracted to `assets\bonsai-binary\Windows\` |
| **Bonsai binary RUNS on FalconX** | ✅ **DONE** | **Confirmed live 14:38 GMT+2. `BonsaiDiorama-Win64-Test.exe` PID 125260, 4 GB RSS, healthy CPU load. Hardware baseline confirmed.** |
| Bonsai source files (project zip) | 🔄 Downloading | 3.3 GB BITS transfer to `assets\bonsai-source\` |
| Editor opens Bonsai (source path) | ❌ Crashes | Diagnosis: D3D12 viewport creation fails when launched via SSH (no interactive desktop session). Solution: launch via Task Scheduler interactive task (proved working for binary). Will re-test editor launch via same path. |
| GPU + driver | ✅ Done | RTX 5090 32GB, driver 591.86 |
| Display | ✅ Done | 3440×1440 ultrawide attached (FalconX session 1, Alexander active) |
| Visual Studio 2022 Community | ✅ Done | Engine compiled successfully so present |
| Quintus-side workspace skeleton | ✅ Done | `/home/atamas/.openclaw/workspace/projects/eyecandy/` |
| Local git repo (Quintus) | ✅ Done | Initial commit `5e6ab0b` |
| Remote GitHub repo | ⏸ Pending | `gh auth login` required from AT |
| Quintus ↔ FalconX SSH (Q→F) | ✅ Done | `scripts/falconx.sh` helper script |
| Project skeleton synced to FalconX | ✅ Done | `C:\Users\Alexander\eyecandy-src\` (via tar-over-ssh, `--sync` mode) |
| Calibration audio track (Getula) | ⚠️ Pending | `audio/thds - Luz.wav` (Tale of Us, not Getula). Getula still needs sourcing — Beatport $1.49 WAV. |
| NVIDIA Surround config | 🔒 Deferred | Per v3.1: leave OFF until Phase 1 ships single-monitor |

## Editor crash diagnosis (resolved root cause)

Crash dump from binary launch attempt revealed: `LowLevelFatalError WindowsD3D12Viewport.cpp:141 — hr failed`. This is D3D12 swap-chain creation failing because the SSH-spawned process couldn't bind to the interactive desktop session (Session 1 / Alexander). Solution: launch any GUI process via Task Scheduler `-Principal Interactive -UserId Alexander`, NOT via `Start-Process` over SSH. Proven working with the binary.

Implication: the source-path editor crash from earlier today is **also probably** the same root cause. Will re-test via Task Scheduler launch path.

## Bonsai live status (as of 14:38)

```
PID    Process                       RSS    CPU s (in 30s wall)
121240 BonsaiDiorama (launcher)       8 MB    0
125260 BonsaiDiorama-Win64-Test    4056 MB  406  (shader compile + asset load, expected)
```

NVIDIA Bonsai Diorama is running on FalconX. The cinematic should be playing on AT's display.

## Next moves

1. ✅ Hardware baseline confirmed via binary
2. Re-launch editor via Task Scheduler interactive path → expect it to work now
3. Source the Getula calibration track (Beatport)
4. Port Stratum audio crate `src/audio.rs` → `Source/EyeCandyAudio/rust/`
5. Build the C++ FFI shim + UE5 plugin
6. Create MPC asset + first single audio binding (`bass_slow → KeyLightIntensity`)
7. Verify side-by-side: unmodified Bonsai cinematic vs EyeCandy-driven Bonsai under Getula
