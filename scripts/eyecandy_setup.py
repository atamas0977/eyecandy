"""
EyeCandy Phase 1 in-editor setup (v2 — uses get_editor_property/set_editor_property).

Run via UnrealEditor-Cmd.exe with -run=pythonscript.
"""

import unreal


def make_mpc():
    path = "/Game/MPC_EyeCandy"
    name = "MPC_EyeCandy"
    eal = unreal.EditorAssetLibrary

    if eal.does_asset_exist(path):
        unreal.log("MPC_EyeCandy already exists, loading...")
        mpc = eal.load_asset(path)
    else:
        atools = unreal.AssetToolsHelpers.get_asset_tools()
        factory = unreal.MaterialParameterCollectionFactoryNew()
        mpc = atools.create_asset(
            asset_name=name,
            package_path="/Game",
            asset_class=unreal.MaterialParameterCollection,
            factory=factory,
        )
        unreal.log(f"Created MPC asset: {mpc.get_path_name()}")

    # Access ScalarParameters via get_editor_property (UE5 Python idiom).
    # The UPROPERTY in MaterialParameterCollection.h is `ScalarParameters`.
    scalars = mpc.get_editor_property("ScalarParameters") or []
    existing_names = [str(s.get_editor_property("ParameterName")) for s in scalars]
    unreal.log(f"Existing scalars: {existing_names}")

    if "KeyLightIntensity" not in existing_names:
        new_entry = unreal.CollectionScalarParameter()
        new_entry.set_editor_property("ParameterName", "KeyLightIntensity")
        new_entry.set_editor_property("DefaultValue", 1.0)
        new_list = list(scalars) + [new_entry]
        mpc.set_editor_property("ScalarParameters", new_list)
        unreal.log("Added KeyLightIntensity scalar (default 1.0)")
    else:
        unreal.log("KeyLightIntensity already present")

    eal.save_asset(mpc.get_path_name(), only_if_is_dirty=False)
    unreal.log("MPC saved")
    return mpc


def tag_key_light():
    level_path = "/Game/Levels/Bonsai_Diorama"
    els = unreal.EditorLevelLibrary

    if not unreal.EditorAssetLibrary.does_asset_exist(level_path):
        unreal.log_warning(f"Level not found: {level_path}")
        return None

    els.load_level(level_path)
    unreal.log(f"Loaded level: {level_path}")

    all_actors = els.get_all_level_actors()
    dl_actors = [a for a in all_actors if isinstance(a, unreal.DirectionalLight)]
    unreal.log(f"Found {len(dl_actors)} DirectionalLight actor(s)")

    if not dl_actors:
        unreal.log_warning("No DirectionalLight in level; cannot tag key light.")
        return None

    target = dl_actors[0]
    existing_tags = [str(t) for t in target.get_editor_property("Tags")]
    label = target.get_actor_label()
    unreal.log(f"Target light: {label}, current tags: {existing_tags}")

    if "EyeCandyKey" not in existing_tags:
        new_tags = list(target.get_editor_property("Tags")) + [unreal.Name("EyeCandyKey")]
        target.set_editor_property("Tags", new_tags)
        unreal.log(f"Tagged {label} with EyeCandyKey")
    else:
        unreal.log("Tag already present")

    # Surface base intensity for sanity
    comp = target.get_component_by_class(unreal.DirectionalLightComponent)
    if comp:
        base_intensity = comp.get_editor_property("Intensity")
        unreal.log(f"Base directional light intensity: {base_intensity}")

    els.save_current_level()
    unreal.log("Level saved")
    return target


def main():
    unreal.log("=== EyeCandy setup START ===")
    make_mpc()
    tag_key_light()
    unreal.log("=== EyeCandy setup DONE ===")


main()
