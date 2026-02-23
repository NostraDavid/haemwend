import json
import sys
from pathlib import Path

import bpy
from mathutils import Vector


def parse_blender_argv(argv=None):
    if argv is None:
        argv = sys.argv
    if "--" in argv:
        return argv[argv.index("--") + 1 :]
    return []


def add_common_output_args(parser):
    parser.add_argument(
        "--render",
        type=str,
        default=None,
        help="Optional output path for rendered image.",
    )
    parser.add_argument(
        "--report-json",
        type=str,
        default=None,
        help="Optional output path for JSON scene report.",
    )
    parser.add_argument(
        "--validate",
        action="store_true",
        help="Validate geometry; exits non-zero on failure.",
    )
    return parser


def ensure_object_mode():
    obj = bpy.context.active_object
    if obj and obj.mode != "OBJECT":
        bpy.ops.object.mode_set(mode="OBJECT")


def get_or_create_collection(name):
    collection = bpy.data.collections.get(name)
    if collection is None:
        collection = bpy.data.collections.new(name)
        bpy.context.scene.collection.children.link(collection)
    return collection


def clear_collection(collection):
    for obj in list(collection.objects):
        bpy.data.objects.remove(obj, do_unlink=True)


def move_to_collection(obj, collection):
    for user_collection in list(obj.users_collection):
        user_collection.objects.unlink(obj)
    collection.objects.link(obj)


def create_cube(name, location, dimensions, collection):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=location)
    obj = bpy.context.active_object
    obj.name = name
    # With primitive_cube_add(size=1.0), object scale maps directly to world dimensions.
    obj.scale = dimensions
    move_to_collection(obj, collection)
    for poly in obj.data.polygons:
        poly.use_smooth = False
    return obj


def ensure_camera_and_light(target):
    scene = bpy.context.scene

    if scene.camera is None:
        camera_data = bpy.data.cameras.new("LLM_Camera")
        camera_obj = bpy.data.objects.new("LLM_Camera", camera_data)
        scene.collection.objects.link(camera_obj)
        scene.camera = camera_obj
    else:
        camera_obj = scene.camera

    camera_obj.location = Vector((3.4, -3.4, 2.5))
    direction = Vector(target) - camera_obj.location
    camera_obj.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()

    key_light = bpy.data.objects.get("LLM_KeyLight")
    if key_light is None:
        light_data = bpy.data.lights.new(name="LLM_KeyLight", type="AREA")
        key_light = bpy.data.objects.new(name="LLM_KeyLight", object_data=light_data)
        scene.collection.objects.link(key_light)
    key_light.location = Vector((2.5, -2.0, 3.0))
    key_light.data.energy = 600.0
    key_light.data.size = 2.0


def maybe_render(path):
    if not path:
        return
    scene = bpy.context.scene
    scene.render.filepath = str(path)
    scene.render.resolution_x = 1024
    scene.render.resolution_y = 1024
    scene.render.image_settings.file_format = "PNG"
    bpy.ops.render.render(write_still=True)


def maybe_write_report(path, report):
    if not path:
        return
    out_path = Path(path)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
