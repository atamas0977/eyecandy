// Copyright (c) 2026 Alexander Tamas. Personal-only.
#pragma once

#include "CoreMinimal.h"
#include "Modules/ModuleManager.h"
#include "Subsystems/EngineSubsystem.h"
#include "EyeCandyAudio.generated.h"

/**
 * Blueprint-readable audio feature snapshot. Mirrors C-FFI EcaFeatures.
 * Polled each tick from the EyeCandyAudio subsystem; readable from Blueprint
 * and material parameter collection drivers.
 */
USTRUCT(BlueprintType)
struct EYECANDYAUDIO_API FEyeCandyAudioFeatures
{
	GENERATED_BODY()

	UPROPERTY(BlueprintReadOnly, Category="EyeCandy|Audio") float BassFast = 0.0f;
	UPROPERTY(BlueprintReadOnly, Category="EyeCandy|Audio") float BassSlow = 0.0f;
	UPROPERTY(BlueprintReadOnly, Category="EyeCandy|Audio") float MidFast  = 0.0f;
	UPROPERTY(BlueprintReadOnly, Category="EyeCandy|Audio") float MidSlow  = 0.0f;
	UPROPERTY(BlueprintReadOnly, Category="EyeCandy|Audio") float TrebleFast = 0.0f;
	UPROPERTY(BlueprintReadOnly, Category="EyeCandy|Audio") float TrebleSlow = 0.0f;

	UPROPERTY(BlueprintReadOnly, Category="EyeCandy|Audio") float KickEnvelope = 0.0f;
	UPROPERTY(BlueprintReadOnly, Category="EyeCandy|Audio") float AudioEnergy  = 0.0f;
	UPROPERTY(BlueprintReadOnly, Category="EyeCandy|Audio") float OnsetEnvelope = 0.0f;
	UPROPERTY(BlueprintReadOnly, Category="EyeCandy|Audio") float BpmEstimate  = 0.0f;

	// Chroma 0..11 — exposed as 12 individual floats for Blueprint friendliness.
	UPROPERTY(BlueprintReadOnly, Category="EyeCandy|Audio|Chroma") TArray<float> Chroma;

	UPROPERTY(BlueprintReadOnly, Category="EyeCandy|Audio|Pos") FVector BassPos = FVector::ZeroVector;
	UPROPERTY(BlueprintReadOnly, Category="EyeCandy|Audio|Pos") FVector MidPos  = FVector::ZeroVector;
	UPROPERTY(BlueprintReadOnly, Category="EyeCandy|Audio|Pos") FVector TreblePos = FVector::ZeroVector;
};

/**
 * Engine subsystem that owns the Rust audio capture lifecycle (eca_init /
 * eca_shutdown) and snapshots features every engine tick.
 */
UCLASS()
class EYECANDYAUDIO_API UEyeCandyAudioSubsystem : public UEngineSubsystem
{
	GENERATED_BODY()
public:
	virtual void Initialize(FSubsystemCollectionBase& Collection) override;
	virtual void Deinitialize() override;

	/** Latest snapshot of audio features. Updated each frame. */
	UFUNCTION(BlueprintCallable, BlueprintPure, Category="EyeCandy|Audio")
	const FEyeCandyAudioFeatures& GetFeatures() const { return CachedFeatures; }

	/** Library version string from the Rust crate, for diagnostics. */
	UFUNCTION(BlueprintCallable, BlueprintPure, Category="EyeCandy|Audio")
	FString GetLibraryVersion() const;

	/** True if the Rust capture initialised successfully. */
	UFUNCTION(BlueprintCallable, BlueprintPure, Category="EyeCandy|Audio")
	bool IsCaptureActive() const { return bCaptureActive; }

private:
	bool TickFeatures(float DeltaTime);

	FEyeCandyAudioFeatures CachedFeatures;
	FTSTicker::FDelegateHandle TickHandle;
	void* DllHandle = nullptr;
	bool bCaptureActive = false;

	// Phase 1 hard-coded binding: BassSlow -> MPC_EyeCandy.KeyLightIntensity.
	// Looked up by soft path on first tick after world init; held weak so we
	// don't keep the asset loaded against PIE world tear-down.
	class UMaterialParameterCollection* CachedMPC = nullptr;
	bool bMPCLookupAttempted = false;

	// Phase 1 hard-coded binding: BassSlow -> DirectionalLightComponent->Intensity
	// for any DirectionalLight actor tagged "EyeCandyKey". We cache the base
	// intensity on first sight so each frame is a multiplicative scaling, not
	// an additive drift. Cleared whenever the world changes.
	TMap<TWeakObjectPtr<class UDirectionalLightComponent>, float> KeyLightBaseIntensity;
};

class FEyeCandyAudioModule : public IModuleInterface
{
public:
	virtual void StartupModule() override;
	virtual void ShutdownModule() override;
};
