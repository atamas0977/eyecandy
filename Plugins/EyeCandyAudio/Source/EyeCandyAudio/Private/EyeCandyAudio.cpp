// Copyright (c) 2026 Alexander Tamas. Personal-only.
#include "EyeCandyAudio.h"
#include "Containers/Ticker.h"
#include "HAL/PlatformProcess.h"
#include "Interfaces/IPluginManager.h"
#include "Misc/Paths.h"
#include "Modules/ModuleManager.h"

DEFINE_LOG_CATEGORY_STATIC(LogEyeCandyAudio, Log, All);

// ---------- C ABI mirror of Rust EcaFeatures ----------
// Layout MUST match Source/EyeCandyAudio/cpp/eyecandy_audio.h exactly.
#pragma pack(push, 4)
struct EcaFeaturesNative
{
	float bass_fast, bass_slow;
	float mid_fast, mid_slow;
	float treble_fast, treble_slow;
	float kick_envelope, audio_energy;
	float onset_envelope, bpm_estimate;
	float chroma[12];
	float bass_pos[3];
	float mid_pos[3];
	float treble_pos[3];
	float _pad[1];
};
#pragma pack(pop)

typedef int32 (*eca_init_fn)(void);
typedef int32 (*eca_get_features_fn)(EcaFeaturesNative*);
typedef int32 (*eca_shutdown_fn)(void);
typedef const char* (*eca_version_fn)(void);

static eca_init_fn         g_eca_init = nullptr;
static eca_get_features_fn g_eca_get_features = nullptr;
static eca_shutdown_fn     g_eca_shutdown = nullptr;
static eca_version_fn      g_eca_version = nullptr;

void UEyeCandyAudioSubsystem::Initialize(FSubsystemCollectionBase& Collection)
{
	Super::Initialize(Collection);

	// Resolve the Rust DLL from the plugin's ThirdParty dir.
	const TSharedPtr<IPlugin> Plugin = IPluginManager::Get().FindPlugin(TEXT("EyeCandyAudio"));
	if (!Plugin.IsValid())
	{
		UE_LOG(LogEyeCandyAudio, Error, TEXT("Plugin not found via IPluginManager"));
		return;
	}
	const FString DllPath = FPaths::Combine(Plugin->GetBaseDir(), TEXT("ThirdParty"), TEXT("EyeCandyAudio"), TEXT("Win64"), TEXT("eyecandy_audio.dll"));
	DllHandle = FPlatformProcess::GetDllHandle(*DllPath);
	if (!DllHandle)
	{
		UE_LOG(LogEyeCandyAudio, Error, TEXT("Failed to load DLL: %s"), *DllPath);
		return;
	}

	g_eca_init         = (eca_init_fn)         FPlatformProcess::GetDllExport(DllHandle, TEXT("eca_init"));
	g_eca_get_features = (eca_get_features_fn) FPlatformProcess::GetDllExport(DllHandle, TEXT("eca_get_features"));
	g_eca_shutdown     = (eca_shutdown_fn)     FPlatformProcess::GetDllExport(DllHandle, TEXT("eca_shutdown"));
	g_eca_version      = (eca_version_fn)      FPlatformProcess::GetDllExport(DllHandle, TEXT("eca_version"));

	if (!g_eca_init || !g_eca_get_features || !g_eca_shutdown)
	{
		UE_LOG(LogEyeCandyAudio, Error, TEXT("Failed to resolve required eca_* symbols"));
		FPlatformProcess::FreeDllHandle(DllHandle);
		DllHandle = nullptr;
		return;
	}

	const int32 InitResult = g_eca_init();
	if (InitResult != 0)
	{
		UE_LOG(LogEyeCandyAudio, Error, TEXT("eca_init returned %d"), InitResult);
		return;
	}
	bCaptureActive = true;

	if (g_eca_version)
	{
		const char* V = g_eca_version();
		UE_LOG(LogEyeCandyAudio, Display, TEXT("Loaded: %s"), V ? UTF8_TO_TCHAR(V) : TEXT("<null>"));
	}

	// Pre-size chroma so Blueprint sees a stable shape.
	CachedFeatures.Chroma.SetNumZeroed(12);

	// Tick every frame on the game thread.
	TickHandle = FTSTicker::GetCoreTicker().AddTicker(
		FTickerDelegate::CreateUObject(this, &UEyeCandyAudioSubsystem::TickFeatures),
		0.0f);
}

void UEyeCandyAudioSubsystem::Deinitialize()
{
	if (TickHandle.IsValid())
	{
		FTSTicker::GetCoreTicker().RemoveTicker(TickHandle);
	}
	if (bCaptureActive && g_eca_shutdown)
	{
		g_eca_shutdown();
		bCaptureActive = false;
	}
	if (DllHandle)
	{
		FPlatformProcess::FreeDllHandle(DllHandle);
		DllHandle = nullptr;
	}
	g_eca_init = nullptr;
	g_eca_get_features = nullptr;
	g_eca_shutdown = nullptr;
	g_eca_version = nullptr;
	Super::Deinitialize();
}

FString UEyeCandyAudioSubsystem::GetLibraryVersion() const
{
	if (g_eca_version)
	{
		const char* V = g_eca_version();
		if (V) { return FString(UTF8_TO_TCHAR(V)); }
	}
	return TEXT("<unloaded>");
}

bool UEyeCandyAudioSubsystem::TickFeatures(float /*DeltaTime*/)
{
	if (!bCaptureActive || !g_eca_get_features) { return true; }

	EcaFeaturesNative N{};
	const int32 R = g_eca_get_features(&N);
	if (R != 0) { return true; }

	CachedFeatures.BassFast = N.bass_fast;
	CachedFeatures.BassSlow = N.bass_slow;
	CachedFeatures.MidFast = N.mid_fast;
	CachedFeatures.MidSlow = N.mid_slow;
	CachedFeatures.TrebleFast = N.treble_fast;
	CachedFeatures.TrebleSlow = N.treble_slow;
	CachedFeatures.KickEnvelope = N.kick_envelope;
	CachedFeatures.AudioEnergy = N.audio_energy;
	CachedFeatures.OnsetEnvelope = N.onset_envelope;
	CachedFeatures.BpmEstimate = N.bpm_estimate;

	if (CachedFeatures.Chroma.Num() != 12) { CachedFeatures.Chroma.SetNumZeroed(12); }
	for (int32 i = 0; i < 12; ++i) { CachedFeatures.Chroma[i] = N.chroma[i]; }

	CachedFeatures.BassPos = FVector(N.bass_pos[0], N.bass_pos[1], N.bass_pos[2]);
	CachedFeatures.MidPos  = FVector(N.mid_pos[0],  N.mid_pos[1],  N.mid_pos[2]);
	CachedFeatures.TreblePos = FVector(N.treble_pos[0], N.treble_pos[1], N.treble_pos[2]);

	return true; // keep ticking
}

// ---------- module ----------

void FEyeCandyAudioModule::StartupModule()
{
	UE_LOG(LogEyeCandyAudio, Display, TEXT("EyeCandyAudio module starting"));
}

void FEyeCandyAudioModule::ShutdownModule()
{
	UE_LOG(LogEyeCandyAudio, Display, TEXT("EyeCandyAudio module shutting down"));
}

IMPLEMENT_MODULE(FEyeCandyAudioModule, EyeCandyAudio)
