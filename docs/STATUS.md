# EyeCandy — Live Status

*Updated 2026-05-12 14:30 GMT+2*

## Phase 0 — Infrastructure Stand-Up

| Item | State | Notes |
|---|---|---|
| NvRTX 5.6 engine built on FalconX | ✅ Done | `C:\Users\Alexander\eyecandy\assets\nvrtx-engine\`, GUID `{6322DDB0-470D-9A69-C2AB-97A502DD2ED5}` registered in HKCU |
| 5 helper binaries built | ✅ Done | SCW, Lightmass, Interchange, CrashReportClient, EpicWebHelper (all in `Engine\Binaries\Win64\`) |
| Bonsai source files present | ✅ Done | `C:\Users\Alexander\eyecandy\zenparticles-ai\BonsaiDiorama\BonsaiDiorama.uproject` |
| Editor opens Bonsai (source path) | ❌ Crashes | Two clean-exit crashes 2026-05-12 14:17 and 14:20; no fatal log lines, no crashdump. Park while we verify hardware via binary path. |
| Bonsai binary download | 🔄 In flight | NVIDIA binary `2025.10.07_BonsaiDioramaDemo_NvRTX_5.6.zip` (1.92 GB) downloading via BITS to `C:\Users\Alexander\eyecandy\assets\bonsai-binary\` |
| GPU + driver | ✅ Done | RTX 5090 32GB, driver 591.86 (>= 581.29 required) |
| Visual Studio 2022 Community | ✅ Done | C++ desktop + Game Dev workloads (assumed; build succeeded so present) |
| Quintus-side workspace skeleton | ✅ Done | `/home/atamas/.openclaw/workspace/projects/eyecandy/` initialized 2026-05-12 14:26 |
| Local git repo (Quintus) | 🔄 Initializing | This commit |
| Remote GitHub repo | ⏸ Pending | `gh auth login` required from AT |
| Quintus ↔ FalconX SSH (Q→F) | ✅ Done | `ssh -i ~/.ssh/falconx_ed25519 Alexander@192.168.0.145` |
| Quintus ↔ FalconX SSH (F→Q) | ⏸ Pending | Not yet needed |
| Calibration audio track | ⚠️ Partial | `audio/thds - Luz.wav` is "THDS - LUZ" (Tale of Us, not Getula). Getula still needs sourcing — Beatport $1.49 |
| NVIDIA Surround config | 🔒 Deferred | Per v3.1: leave OFF until Phase 1 ships single-monitor |

## Open editor-crash diagnosis

Two source-path editor launches crashed without writing a fatal log entry:
- Launch 1: 2026-05-11 22:16 → real error (`Couldn't launch ShaderCompileWorker.exe!`) because helpers weren't yet built. Helpers build kicked off and finished 2026-05-12 00:19.
- Launch 2: 2026-05-12 14:20 → no log of failure, log ends mid-Streaming-Display chatter at `LogTurnkeySupport: Completed SDK detection: ExitCode = 1`, then silent process death. No crash report under `Saved\Crashes\`.

Hypotheses (to investigate after Phase 0 visual baseline closes):
1. Streamline DLSS NGX cache miss (`Feature dlss failed to load`, `Feature dlssg failed to load` — both fall back to driver, may still cause downstream init issue)
2. CrashReportClient living in `Win64\` root instead of expected `Win64\CrashReportClient\` subdirectory — engine may try to chain-launch CRC and fail before fatal-log handler
3. `aqProf.dll` / `VtuneApi.dll` / `WinPixGpuCapturer.dll` all missing (profilers — should be benign warnings, but tagged)
4. SSH-launched process inheriting wrong WindowStation / Session — try interactive RDP launch as control

## Next moves (locked, executing now)

1. Wait for Bonsai binary download to complete (~3-5 min on a reasonable WAN, 1.92 GB)
2. Extract + launch the binary `BonsaiDiorama.exe` on FalconX
3. **If binary runs** → hardware baseline confirmed, source-build editor crash is a known investigation, parked
4. **If binary also crashes** → driver/Surround/Windows issue, deeper debug

In parallel: git skeleton (done), audio crate port from Stratum (queued).
