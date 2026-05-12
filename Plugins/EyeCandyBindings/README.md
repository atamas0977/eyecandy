# Plugin: EyeCandyBindings

UE5 plugin that wires the audio features (from `EyeCandyAudio`) into scene parameters:

- **Material Parameter Collection (MPC) driver** — for each binding declared in a `UDataAsset`, on tick, read the named audio feature, optionally apply a curve, and write the scalar to a target MPC.
- **Light intensity driver** — direct binding of audio features to directional/point light intensities.
- **6DoF spectator pawn** — Phase 1 §VI of v3.1: Tab pauses cinematic, hands control to a freecam pawn.
- **Cinematic state machine** — manages cinematic ↔ spectator transitions (camera transform preservation, audio bindings stay live in both modes).

## Default Phase 1 bindings (Bonsai)

Codified in `Content/EyeCandy/DataAssets/DA_BonsaiBindings.uasset` (planned):

| Source | Target | Range | Curve |
|---|---|---|---|
| `bass_slow` | MPC `KeyLightIntensity` | 0.7× → 1.6× base | linear |
| `audio_energy` (slow integ) | MPC `TimeOfDayPhase` | 0.0 → 1.0 | linear |
| `mid_slow` | MPC `FogDensity` | 0.2× → 1.4× base | linear |
| `treble_fast` + `kick_envelope` | MPC `GlassGlintEmissive` | 0.0 → 2.0 | refractory-gated additive |

## Status

⏸ Not yet implemented. Phase 1 task #3 (depends on EyeCandyAudio plugin).
