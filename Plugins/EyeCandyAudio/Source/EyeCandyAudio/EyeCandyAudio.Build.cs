// Copyright (c) 2026 Alexander Tamas. Personal-only.
using System.IO;
using UnrealBuildTool;

public class EyeCandyAudio : ModuleRules
{
	public EyeCandyAudio(ReadOnlyTargetRules Target) : base(Target)
	{
		PCHUsage = ModuleRules.PCHUsageMode.UseExplicitOrSharedPCHs;

		PublicIncludePaths.AddRange(new string[] {
			Path.Combine(ModuleDirectory, "Public"),
		});

		PrivateIncludePaths.AddRange(new string[] {
			Path.Combine(ModuleDirectory, "Private"),
		});

		PublicDependencyModuleNames.AddRange(new string[] {
			"Core",
			"CoreUObject",
			"Engine",
		});

		PrivateDependencyModuleNames.AddRange(new string[] {
			"Projects",  // for IPluginManager / module log routing
		});

		// Bundle the Rust cdylib alongside the UE5 plugin.
		// The .dll is produced by `cargo build --release` in
		// projects/eyecandy/Source/EyeCandyAudio/rust/ and copied here as part
		// of the cross-machine sync. See scripts/sync_rust_dll.sh.
		if (Target.Platform == UnrealTargetPlatform.Win64)
		{
			string DllDir = Path.Combine(PluginDirectory, "ThirdParty", "EyeCandyAudio", "Win64");
			string DllPath = Path.Combine(DllDir, "eyecandy_audio.dll");

			RuntimeDependencies.Add(DllPath);
			PublicDelayLoadDLLs.Add("eyecandy_audio.dll");
			PublicAdditionalLibraries.Add(Path.Combine(DllDir, "eyecandy_audio.dll.lib"));

			// C header (mirrors the Rust struct layout) lives next to the DLL.
			PublicIncludePaths.Add(DllDir);
		}
	}
}
