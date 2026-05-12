# Build Notes — NvRTX 5.6 on FalconX

*From memory/2026-05-11.md 23:30 entry. Preserved as authoritative reference for future engine rebuilds.*

## Hard-won lessons from the 2026-05-11/12 build

1. **Pagefile must be system-managed** before any UE5 source build. With a fixed pagefile, build OOMs early with `C1076: compiler limit: internal heap limit reached`.
   ```powershell
   (Get-WmiObject Win32_ComputerSystem).AutomaticManagedPagefile = $true
   ```

2. **Use `-NoXGE`** unless you have a licensed XGE installation. The build's default XGE accelerator causes intermittent corrupt-COFF errors (`CVT1107` on `.obj` files). On retry, wipe any half-written `Intermediate\Build\Win64\x64\UnrealEditor\Development\<Module>\` directory.

3. **`Build.bat UnrealEditor` is NOT a full build.** Builds the editor + DLL graph but skips helper programs. The editor then fails at runtime with "Unable to launch ShaderCompileWorker" on project open. Required helper targets:
   - `ShaderCompileWorker` (Development, Win64) — critical
   - `UnrealLightmass` (Development, Win64)
   - `InterchangeWorker` (Development, Win64)
   - `CrashReportClient` (**Shipping**, Win64)
   - `EpicWebHelper` (Development, Win64)

4. **`UnrealVersionSelector` ships as `UnrealVersionSelector-Win64-Shipping.exe`** — must be renamed/copied to `UnrealVersionSelector.exe`, then run with `/register` flag to register the engine GUID into HKCU. Bonsai's `.uproject` references the engine by GUID, so without this step the project won't bind.

5. **CrashReportClient binary lives in `Win64\` root**, not in `Win64\CrashReportClient\` subdirectory as the NvRTX 5.6 helper-build-watcher cron assumed. The cron's path-existence check returned false negatives; binary IS there at `Win64\CrashReportClient.exe`.

## Engine GUID (current build)

```
{6322DDB0-470D-9A69-C2AB-97A502DD2ED5}
```

Registered at HKCU\Software\Epic Games\Unreal Engine\Builds.

## Editor crash investigation (open)

Editor crashes silently on Bonsai source-path open. See STATUS.md "Open editor-crash diagnosis" for hypotheses. Resolution path: validate hardware via NVIDIA binary distribution first, then return to source-build crash with hardware confirmed.
