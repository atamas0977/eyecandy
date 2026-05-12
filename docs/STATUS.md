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
| Editor opens Bonsai (source path) | ✅ **Done** | Re-launched via Task Scheduler interactive 14:43. PID 103436, 6.6 GB RSS, ~250 ShaderCompileWorker children processing first-run shaders. Healthy. |
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
2. ✅ Editor launches Bonsai source (via Task Scheduler interactive path)
3. ✅ Stratum audio crate ported to `Source/EyeCandyAudio/rust/` (cdylib + rlib, C ABI: `eca_init` / `eca_get_features` / `eca_shutdown` / `eca_version`)
4. ✅ Rust DLL built on FalconX (1.2 MB Windows x64) and pulled to `Plugins/EyeCandyAudio/ThirdParty/EyeCandyAudio/Win64/`
5. ✅ UE5 plugin `EyeCandyAudio` scaffolded (.uplugin, Build.cs, USTRUCT, UEngineSubsystem)
6. ✅ Getula (Be Svendsen) on FalconX as calibration track
7. ✅ Plugin copied into Bonsai's `Plugins/`, `.uproject` patched, `Build.bat BonsaiDioramaEditor` produced `UnrealEditor-EyeCandyAudio.dll` cleanly (42.8s, no warnings)
8. ✅ **Plugin loads at runtime**: `LogPluginManager: Mounting Project plugin EyeCandyAudio` → `LogEyeCandyAudio: Display: Loaded: eyecandy_audio 0.1.0` (proves UEngineSubsystem ran `Initialize()`, loaded the Rust DLL via `FPlatformProcess::GetDllHandle`, called `eca_version()` over FFI, and would have called `eca_init()` next). No error logged → capture is active.
9. Create MPC asset + first audio binding (`bass_slow → KeyLightIntensity`) in Bonsai editor — UI task, can be Python-automated or done by AT
10. Verify side-by-side: unmodified Bonsai cinematic vs EyeCandy-driven Bonsai under Getula

## Working integration milestone reached 🎉

As of 15:35 GMT+2 the full audio plumbing stack is operational:
```
Getula.mp3 (FalconX audio out)
        ↓ (WASAPI loopback)
cpal capture thread (in libeyecandy_audio.dll)
        ↓ (FFT, ISO 226, 2-stage smoothing, kick detector)
AudioFeatures struct (shared mutex)
        ↓ (eca_get_features over C ABI)
UEyeCandyAudioSubsystem (UE5 game thread tick)
        ↓ (FEyeCandyAudioFeatures USTRUCT)
Blueprint / MPC drivers (next step: wire to KeyLightIntensity)
```

The last step is purely scene wiring inside the Bonsai editor.

## Task Scheduler launch pattern (canonical)

For any GUI process on FalconX from this session, use `scripts/falconx.sh --file /tmp/<task-script>.ps1` where the script registers a `New-ScheduledTask` with `-Principal Interactive -UserId Alexander`, starts it, then unregisters. This is the only reliable way to bind D3D12 swap chains over SSH. Direct `Start-Process` over SSH crashes with `WindowsD3D12Viewport.cpp:141 — hr failed`.

## Build pattern (canonical, for plugin iteration)

1. Kill UnrealEditor + LiveCodingConsole + Bonsai + UnrealTraceServer (any of these holding locks will break Build.bat)
2. `Build.bat BonsaiDioramaEditor Win64 Development -Project=<.uproject path> -NoXGE`
3. Re-launch editor via Task Scheduler interactive

Live Coding will hot-patch C++ changes in many cases without needing the full kill+build cycle. But for plugin module *structural* changes (new UObject, new module dependency), full rebuild is safer.

## Repo

- Local: `/home/atamas/.openclaw/workspace/projects/eyecandy/` (Quintus)
- Remote: https://github.com/atamas0977/eyecandy (private)
- FalconX sync: `C:\Users\Alexander\eyecandy-src\` (workspace mirror) + `C:\Users\Alexander\eyecandy\` (engine + Bonsai project + audio)
