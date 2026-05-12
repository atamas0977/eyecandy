"""Create a minimal test level for EyeCandy binding verification.

The level contains:
  - DirectionalLight (tagged 'EyeCandyKey', base intensity 3.0)
  - SkyAtmosphere (for visual context)
  - A floor plane (for the light to land on)
  - A cube (for shadows/specular sense)
  - A SkyLight (very dim, so the directional drives most of the visible light)
  - A default camera position

Loads in seconds. Used to verify the audio->light binding visibly while
Bonsai's heavy Nanite source is still building.
"""

import unreal


LEVEL_PATH = "/Game/Levels/EyeCandyTest"


def main():
    unreal.log("=== EyeCandyTest level setup ===")
    eal = unreal.EditorAssetLibrary
    els = unreal.EditorLevelLibrary
    eas = unreal.EditorActorSubsystem

    actor_sub = unreal.get_editor_subsystem(unreal.EditorActorSubsystem)
    level_sub = unreal.get_editor_subsystem(unreal.LevelEditorSubsystem)

    # Create new empty level
    if eal.does_asset_exist(LEVEL_PATH):
        unreal.log(f"Level {LEVEL_PATH} exists; loading.")
        level_sub.load_level(LEVEL_PATH)
    else:
        unreal.log(f"Creating new level: {LEVEL_PATH}")
        level_sub.new_level(LEVEL_PATH)

    # Wipe existing actors (idempotent re-runs)
    all_actors = actor_sub.get_all_level_actors()
    removable_classes = (unreal.DirectionalLight, unreal.SkyLight, unreal.SkyAtmosphere,
                         unreal.StaticMeshActor, unreal.CameraActor)
    for a in all_actors:
        if isinstance(a, removable_classes):
            actor_sub.destroy_actor(a)

    # 1. Directional light, tagged
    dl = actor_sub.spawn_actor_from_class(
        unreal.DirectionalLight, unreal.Vector(0, 0, 500), unreal.Rotator(-40, 30, 0))
    dl.set_actor_label("KeyLight")
    dl.set_editor_property("Tags", [unreal.Name("EyeCandyKey")])
    dl_comp = dl.get_component_by_class(unreal.DirectionalLightComponent)
    dl_comp.set_editor_property("Intensity", 3.0)
    dl_comp.set_editor_property("LightColor", unreal.LinearColor(1.0, 0.95, 0.85, 1.0).to_color())
    unreal.log(f"Spawned KeyLight at intensity 3.0, tag=EyeCandyKey")

    # 2. SkyAtmosphere for environment
    sky_atmos = actor_sub.spawn_actor_from_class(unreal.SkyAtmosphere, unreal.Vector(0, 0, 0))
    sky_atmos.set_actor_label("SkyAtmosphere")

    # 3. SkyLight (dim, so directional dominates)
    sky_light = actor_sub.spawn_actor_from_class(unreal.SkyLight, unreal.Vector(0, 0, 300))
    sky_light.set_actor_label("SkyLight")
    sl_comp = sky_light.get_component_by_class(unreal.SkyLightComponent)
    sl_comp.set_editor_property("Intensity", 0.4)
    # SkyLight will capture on first tick automatically; manual capture properties
    # vary by UE version, skip them.

    # 4. Floor plane
    floor_mesh = eal.load_asset("/Engine/BasicShapes/Plane.Plane")
    floor = actor_sub.spawn_actor_from_class(
        unreal.StaticMeshActor, unreal.Vector(0, 0, 0), unreal.Rotator(0, 0, 0))
    floor.set_actor_label("Floor")
    floor.set_actor_scale3d(unreal.Vector(50, 50, 1))
    floor.static_mesh_component.set_static_mesh(floor_mesh)
    # Default material from engine is fine

    # 5. Hero cube
    cube_mesh = eal.load_asset("/Engine/BasicShapes/Cube.Cube")
    cube = actor_sub.spawn_actor_from_class(
        unreal.StaticMeshActor, unreal.Vector(0, 0, 100), unreal.Rotator(0, 0, 0))
    cube.set_actor_label("HeroCube")
    cube.set_actor_scale3d(unreal.Vector(2, 2, 2))
    cube.static_mesh_component.set_static_mesh(cube_mesh)

    # 6. Camera positioned to see cube + floor
    cam = actor_sub.spawn_actor_from_class(
        unreal.CameraActor, unreal.Vector(-600, 0, 400), unreal.Rotator(-15, 0, 0))
    cam.set_actor_label("MainCamera")

    # Snap viewport to camera position (best-effort; may no-op in commandlet mode)
    try:
        level_sub.editor_set_game_view(False)
        level_sub.pilot_level_actor(cam)
        level_sub.eject_pilot_level_actor()
    except Exception:
        pass

    # Save level
    level_sub.save_current_level()
    unreal.log(f"Saved level: {LEVEL_PATH}")
    unreal.log("=== EyeCandyTest setup DONE ===")


main()
