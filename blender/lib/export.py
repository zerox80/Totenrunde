"""glTF-Export (als .gltf + .bin; Texturen werden aus assets/textures referenziert)."""

import os

import bpy

def export_gltf(path, animations=False, selection=None, extras=False):
    """Exportiert die Szene als .gltf (+ .bin). Texturen werden aus assets/textures referenziert."""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    kw = dict(
        filepath=path,
        export_format="GLTF_SEPARATE",
        export_yup=True,
        export_apply=True,
        export_texcoords=True,
        export_normals=True,
        export_materials="EXPORT",
        export_keep_originals=True,
        export_animations=animations,
        export_extras=extras,
        export_lights=False,
        export_cameras=False,
    )
    if animations:
        kw.update(
            export_animation_mode="ACTIONS",
            export_force_sampling=True,
            export_anim_single_armature=True,
            export_reset_pose_bones=True,
            export_optimize_animation_size=False,
            export_anim_slide_to_zero=True,
        )
    if selection is not None:
        bpy.ops.object.select_all(action="DESELECT")
        for o in selection:
            o.select_set(True)
        kw["use_selection"] = True
    bpy.ops.export_scene.gltf(**kw)
    print("EXPORTIERT:", path)
