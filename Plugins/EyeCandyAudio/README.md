# Plugin: EyeCandyAudio

UE5 plugin that:
1. Loads the Rust `libeyecandy_audio.dll` (cdylib) via dlopen-style binding at module startup
2. Spawns an audio capture thread on WASAPI loopback (Windows shared-mode loopback)
3. Pushes 1024-sample frames into the Rust analyser, polls the `AudioFeatures` struct each engine tick
4. Exposes a `UEyeCandyAudioSubsystem` (UWorldSubsystem) that publishes the features as Blueprint-readable properties

## Plugin descriptor (planned)

`EyeCandyAudio.uplugin`:
- Category: `Audio`
- LoadingPhase: `PreDefault` (need it up before MPCs are loaded)
- Modules: `EyeCandyAudio` (Runtime)
- Dependencies: `Core`, `CoreUObject`, `Engine`, `AudioMixer`

## Why a plugin, not a project module

So Bonsai (or any future scene project) can pick it up via `.uproject` `Plugins` array without copying source.

## Status

⏸ Not yet implemented. Phase 1 task #2 (after binary baseline confirmed).
