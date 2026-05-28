bl_info = {
    "name": "LLM Stylized Tree Generator",
    "author": "NostraDavid + Codex",
    "version": (1, 0, 0),
    "blender": (4, 0, 0),
    "location": "View3D > Sidebar > LLM Trees",
    "description": "Deterministic stylized broadleaf and pine tree generator.",
    "category": "Add Mesh",
}

import argparse
import json
import math
import random
import sys
import time
from dataclasses import asdict, dataclass
from pathlib import Path

import bpy
from mathutils import Vector


COLLECTION_NAME = "LLM_Trees"
ROOT_PROP = "llm_tree_root"
LAST_ROOT_PROP = "llm_tree_last_root"
AUTO_UPDATE_DEADLINE_PROP = "llm_tree_auto_update_deadline"
AUTO_UPDATE_TIMER_PROP = "llm_tree_auto_update_timer_registered"
SUPPRESS_UPDATE_PROP = "llm_tree_suppress_auto_update"


@dataclass(frozen=True)
class TreeParams:
    seed: int = 1234
    tree_type: str = "broadleaf"
    height: float = 4.2
    trunk_radius: float = 0.24
    trunk_bend: float = 0.35
    trunk_taper: float = 0.42
    branch_count: int = 7
    branch_spread: float = 1.15
    clump_count: int = 9
    crown_radius: float = 1.35
    crown_height: float = 1.45
    asymmetry: float = 0.42
    exaggeration: float = 0.65
    chunky_silhouette: float = 0.72
    color_variation: float = 0.30
    bark_hue: float = 0.075
    foliage_hue: float = 0.28
    show_scale_reference: bool = True
    scale_reference_distance: float = 0.8
    placement_index: int = -1


@dataclass(frozen=True)
class TreeReport:
    root_name: str
    collection: str
    seed: int
    tree_type: str
    parameters: dict
    objects: list
    mesh_objects: int
    poly_count: int
    bbox_min: list
    bbox_max: list


def _parse_blender_argv(argv=None):
    argv = sys.argv if argv is None else argv
    if "--" in argv:
        return argv[argv.index("--") + 1 :]
    return []


def _ensure_object_mode():
    obj = bpy.context.active_object
    if obj and obj.mode != "OBJECT":
        bpy.ops.object.mode_set(mode="OBJECT")


def _clear_selection():
    for obj in bpy.context.selected_objects:
        obj.select_set(False)


def _get_collection():
    collection = bpy.data.collections.get(COLLECTION_NAME)
    if collection is None:
        collection = bpy.data.collections.new(COLLECTION_NAME)
        bpy.context.scene.collection.children.link(collection)
    return collection


def _move_to_collection(obj, collection):
    for user_collection in list(obj.users_collection):
        user_collection.objects.unlink(obj)
    collection.objects.link(obj)


def _count_existing_roots(collection):
    return sum(1 for obj in collection.objects if obj.get(ROOT_PROP))


def _next_placement_index(collection):
    used = []
    for obj in collection.objects:
        if obj.get(ROOT_PROP):
            used.append(int(obj.get("placement_index", 0)))
    return max(used, default=-1) + 1


def _hsv_to_rgb(h, s, v):
    import colorsys

    r, g, b = colorsys.hsv_to_rgb(h % 1.0, max(0.0, min(1.0, s)), max(0.0, min(1.0, v)))
    return (r, g, b, 1.0)


def _make_material(name, color, roughness=0.86):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs["Base Color"].default_value = color
        bsdf.inputs["Roughness"].default_value = roughness
    mat.diffuse_color = color
    return mat


def _make_palette(params):
    variation = max(0.0, min(1.0, params.color_variation))
    bark_base = _hsv_to_rgb(params.bark_hue, 0.54 + variation * 0.12, 0.34 + variation * 0.04)
    bark_light = _hsv_to_rgb(params.bark_hue + 0.018 + variation * 0.035, 0.48 + variation * 0.12, 0.50 + variation * 0.12)
    bark_dark = _hsv_to_rgb(params.bark_hue - 0.014 - variation * 0.026, 0.62 + variation * 0.18, 0.20 + variation * 0.03)
    leaf_base = _hsv_to_rgb(params.foliage_hue, 0.66 + variation * 0.16, 0.44 + variation * 0.10)
    leaf_light = _hsv_to_rgb(params.foliage_hue + 0.018 + variation * 0.034, 0.56 + variation * 0.16, 0.60 + variation * 0.16)
    leaf_dark = _hsv_to_rgb(params.foliage_hue - 0.026 - variation * 0.030, 0.76 + variation * 0.18, 0.24 + variation * 0.06)
    return {
        "bark_base": _make_material(f"Tree_Bark_{params.seed}_Base", bark_base),
        "bark_light": _make_material(f"Tree_Bark_{params.seed}_Light", bark_light),
        "bark_dark": _make_material(f"Tree_Bark_{params.seed}_Dark", bark_dark),
        "leaf_base": _make_material(f"Tree_Leaf_{params.seed}_Base", leaf_base),
        "leaf_light": _make_material(f"Tree_Leaf_{params.seed}_Light", leaf_light),
        "leaf_dark": _make_material(f"Tree_Leaf_{params.seed}_Dark", leaf_dark),
    }


def _parent(child, parent):
    child.parent = parent
    child.matrix_parent_inverse = parent.matrix_world.inverted()


def _create_box(name, location, scale, collection, material):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=location)
    obj = bpy.context.active_object
    obj.name = name
    obj.dimensions = scale
    bpy.context.view_layer.update()
    _move_to_collection(obj, collection)
    obj.data.materials.append(material)
    for poly in obj.data.polygons:
        poly.use_smooth = False
    return obj


def _create_scale_reference(root, params, collection):
    material = _make_material("Tree_Scale_Reference_Man", (0.18, 0.18, 0.20, 1.0), roughness=0.9)
    x = params.crown_radius + params.scale_reference_distance
    parts = [
        _create_box("ScaleMan_Torso_180cm", (x, 0.0, 1.17), (0.38, 0.20, 0.68), collection, material),
        _create_box("ScaleMan_Hips_180cm", (x, 0.0, 0.81), (0.32, 0.18, 0.24), collection, material),
        _create_box("ScaleMan_Leg_L_180cm", (x - 0.07, 0.0, 0.40), (0.11, 0.11, 0.80), collection, material),
        _create_box("ScaleMan_Leg_R_180cm", (x + 0.07, 0.0, 0.40), (0.11, 0.11, 0.80), collection, material),
        _create_box("ScaleMan_Arm_L_180cm", (x - 0.26, 0.0, 1.13), (0.09, 0.09, 0.64), collection, material),
        _create_box("ScaleMan_Arm_R_180cm", (x + 0.26, 0.0, 1.13), (0.09, 0.09, 0.64), collection, material),
        _create_box("ScaleMan_Shoulder_180cm", (x, 0.0, 1.48), (0.54, 0.19, 0.11), collection, material),
    ]
    bpy.ops.mesh.primitive_uv_sphere_add(segments=8, ring_count=4, radius=0.125, location=(x, 0.0, 1.675))
    head = bpy.context.active_object
    head.name = "ScaleMan_Head_180cm"
    _move_to_collection(head, collection)
    head.data.materials.append(material)
    for poly in head.data.polygons:
        poly.use_smooth = False
    parts.append(head)

    for obj in parts:
        obj["scale_reference"] = "male_180cm"
        _parent(obj, root)
    return parts


def _make_low_poly_cylinder(name, points, radii, material_slots, collection, sides=7):
    verts = []
    faces = []
    ring_count = len(points)
    for i, (point, radius) in enumerate(zip(points, radii)):
        direction = (points[min(i + 1, ring_count - 1)] - points[max(i - 1, 0)]).normalized()
        if direction.length < 1e-6:
            direction = Vector((0.0, 0.0, 1.0))
        up = Vector((0.0, 0.0, 1.0))
        if abs(direction.dot(up)) > 0.93:
            up = Vector((1.0, 0.0, 0.0))
        right = direction.cross(up).normalized()
        forward = right.cross(direction).normalized()
        twist = i * 0.37
        for s in range(sides):
            angle = (math.tau * s / sides) + twist
            wobble = 1.0 + 0.10 * math.sin(angle * 3.0 + i * 1.7)
            verts.append(point + (right * math.cos(angle) + forward * math.sin(angle)) * radius * wobble)

    for i in range(ring_count - 1):
        for s in range(sides):
            a = i * sides + s
            b = i * sides + ((s + 1) % sides)
            c = (i + 1) * sides + ((s + 1) % sides)
            d = (i + 1) * sides + s
            faces.append((a, b, c, d))
    faces.append(tuple(range(sides - 1, -1, -1)))
    top_start = (ring_count - 1) * sides
    faces.append(tuple(range(top_start, top_start + sides)))

    mesh = bpy.data.meshes.new(name + "Mesh")
    mesh.from_pydata([tuple(v) for v in verts], [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    collection.objects.link(obj)
    for mat in material_slots:
        obj.data.materials.append(mat)
    for i, poly in enumerate(obj.data.polygons):
        poly.use_smooth = False
        if i < sides:
            poly.material_index = 2 if len(material_slots) > 2 else 0
        elif i % 3 == 0 and len(material_slots) > 1:
            poly.material_index = 1
        else:
            poly.material_index = 0
    return obj


def _ellipsoid_mesh(name, location, scale, rng, collection, materials, rings=5, segments=8, squash_noise=0.18):
    verts = []
    faces = []
    for r in range(rings + 1):
        phi = -math.pi / 2.0 + math.pi * r / rings
        ring_radius = math.cos(phi)
        z = math.sin(phi)
        for s in range(segments):
            angle = math.tau * s / segments
            wobble = 1.0 + rng.uniform(-squash_noise, squash_noise)
            verts.append(
                (
                    math.cos(angle) * ring_radius * scale.x * wobble,
                    math.sin(angle) * ring_radius * scale.y * wobble,
                    z * scale.z * (1.0 + rng.uniform(-squash_noise * 0.5, squash_noise * 0.5)),
                )
            )
    for r in range(rings):
        for s in range(segments):
            a = r * segments + s
            b = r * segments + ((s + 1) % segments)
            c = (r + 1) * segments + ((s + 1) % segments)
            d = (r + 1) * segments + s
            faces.append((a, b, c, d))
    mesh = bpy.data.meshes.new(name + "Mesh")
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    obj.location = location
    obj.rotation_euler = (rng.uniform(-0.18, 0.18), rng.uniform(-0.18, 0.18), rng.uniform(0.0, math.tau))
    collection.objects.link(obj)
    for mat in materials:
        obj.data.materials.append(mat)
    for poly in obj.data.polygons:
        poly.use_smooth = False
        center_z = sum(obj.data.vertices[i].co.z for i in poly.vertices) / len(poly.vertices)
        if center_z > scale.z * 0.25:
            poly.material_index = 1
        elif center_z < -scale.z * 0.30:
            poly.material_index = 2
        else:
            poly.material_index = 0
    return obj


def _cone_tier_mesh(name, location, radius, height, rng, collection, materials, segments=8):
    verts = [(0.0, 0.0, height * 0.45)]
    lower_z = -height * 0.38
    mid_z = -height * 0.05
    for z, scale in ((lower_z, 1.0), (mid_z, 0.62)):
        for s in range(segments):
            angle = math.tau * s / segments
            wobble = 1.0 + rng.uniform(-0.16, 0.14)
            verts.append((math.cos(angle) * radius * scale * wobble, math.sin(angle) * radius * scale * wobble, z))
    faces = []
    lower = 1
    mid = 1 + segments
    for s in range(segments):
        faces.append((0, mid + s, mid + ((s + 1) % segments)))
        faces.append((mid + s, lower + s, lower + ((s + 1) % segments), mid + ((s + 1) % segments)))
    faces.append(tuple(range(lower + segments - 1, lower - 1, -1)))
    mesh = bpy.data.meshes.new(name + "Mesh")
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    obj.location = location
    obj.rotation_euler = (rng.uniform(-0.12, 0.12), rng.uniform(-0.12, 0.12), rng.uniform(0.0, math.tau))
    collection.objects.link(obj)
    for mat in materials:
        obj.data.materials.append(mat)
    for poly in obj.data.polygons:
        poly.use_smooth = False
        poly.material_index = 1 if poly.center.z > 0 else 0
    return obj


def _make_branch(name, start, angle, length, radius, rng, params, materials, collection):
    bend = params.exaggeration * rng.uniform(0.05, 0.18)
    end = start + Vector((math.cos(angle) * length, math.sin(angle) * length, length * rng.uniform(0.18, 0.42)))
    mid = (start + end) * 0.5 + Vector((rng.uniform(-bend, bend), rng.uniform(-bend, bend), bend))
    points = [start, mid, end]
    radii = [radius, radius * 0.68, radius * 0.24]
    return _make_low_poly_cylinder(name, points, radii, materials, collection, sides=6)


def _tree_path(params, rng):
    levels = 7
    points = []
    for i in range(levels):
        t = i / (levels - 1)
        bend = params.trunk_bend * (t * t)
        sway = Vector((
            math.sin(t * math.pi * 0.9 + params.seed * 0.013) * bend,
            math.cos(t * math.pi * 0.7 + params.seed * 0.017) * bend * 0.55,
            params.height * t,
        ))
        points.append(sway)
    radii = [
        params.trunk_radius * (1.0 - (1.0 - params.trunk_taper) * (i / (levels - 1))) * (1.0 + 0.06 * math.sin(i * 1.9))
        for i in range(levels)
    ]
    return points, radii


def _create_root(params, collection):
    root_name = f"Tree_Seed_{params.seed}_{params.tree_type.title()}"
    if bpy.data.objects.get(root_name):
        suffix = _count_existing_roots(collection) + 1
        root_name = f"{root_name}_{suffix:03d}"
    root = bpy.data.objects.new(root_name, None)
    root.empty_display_type = "PLAIN_AXES"
    root.empty_display_size = 0.45
    root[ROOT_PROP] = True
    root["seed"] = int(params.seed)
    root["tree_type"] = params.tree_type
    root["placement_index"] = int(params.placement_index)
    collection.objects.link(root)
    root.location.x = params.placement_index * max(params.crown_radius * 2.7, 2.25)
    return root


def create_tree(params=None, replace_existing=True):
    """Create a deterministic stylized tree and return a report dictionary."""
    _ensure_object_mode()
    if params is None:
        params = TreeParams()
    elif isinstance(params, dict):
        params = TreeParams(**params)
    if params.tree_type not in {"broadleaf", "pine"}:
        raise ValueError("tree_type must be 'broadleaf' or 'pine'")
    if params.height <= 0.5:
        raise ValueError("height must be > 0.5")
    if params.trunk_radius <= 0.02:
        raise ValueError("trunk_radius must be > 0.02")

    if replace_existing:
        clear_generated_trees()
    collection = _get_collection()
    if params.placement_index < 0:
        placement_index = 0 if replace_existing else _next_placement_index(collection)
        params = TreeParams(**{**asdict(params), "placement_index": placement_index})
    rng = random.Random(int(params.seed))
    palette = _make_palette(params)
    root = _create_root(params, collection)

    points, radii = _tree_path(params, rng)
    trunk = _make_low_poly_cylinder(
        "Trunk",
        points,
        radii,
        [palette["bark_base"], palette["bark_light"], palette["bark_dark"]],
        collection,
        sides=7,
    )
    _parent(trunk, root)
    generated = [root, trunk]

    if params.tree_type == "broadleaf":
        branch_start = max(2, int(len(points) * 0.35))
        for i in range(max(1, params.branch_count)):
            t = (i + 0.35) / max(1, params.branch_count)
            source_index = min(len(points) - 2, branch_start + int(t * (len(points) - branch_start - 1)))
            start = points[source_index]
            angle = math.tau * t + rng.uniform(-0.38, 0.38)
            length = params.branch_spread * rng.uniform(0.55, 1.05) * (1.0 + params.exaggeration * 0.25)
            radius = params.trunk_radius * rng.uniform(0.18, 0.34)
            branch = _make_branch(f"Branch_{i + 1:02d}", start, angle, length, radius, rng, params, [palette["bark_base"], palette["bark_light"], palette["bark_dark"]], collection)
            _parent(branch, root)
            generated.append(branch)

        crown_center = points[-1] + Vector((0.0, 0.0, -params.crown_height * 0.15))
        leaf_mats = [palette["leaf_base"], palette["leaf_light"], palette["leaf_dark"]]
        for i in range(max(1, params.clump_count)):
            angle = math.tau * i / max(1, params.clump_count) + rng.uniform(-0.45, 0.45)
            radial = params.crown_radius * rng.uniform(0.05, 0.82) * (1.0 + params.asymmetry * rng.uniform(-0.35, 0.55))
            loc = crown_center + Vector((
                math.cos(angle) * radial + rng.uniform(-params.asymmetry, params.asymmetry),
                math.sin(angle) * radial * rng.uniform(0.72, 1.12),
                rng.uniform(-params.crown_height * 0.28, params.crown_height * 0.45),
            ))
            scale = Vector((
                params.crown_radius * rng.uniform(0.42, 0.72) * (1.0 + params.chunky_silhouette * 0.25),
                params.crown_radius * rng.uniform(0.36, 0.64),
                params.crown_height * rng.uniform(0.28, 0.48),
            ))
            clump = _ellipsoid_mesh(f"LeafClump_{i + 1:02d}", loc, scale, rng, collection, leaf_mats)
            _parent(clump, root)
            generated.append(clump)
    else:
        tier_count = max(4, params.clump_count)
        leaf_mats = [palette["leaf_base"], palette["leaf_light"], palette["leaf_dark"]]
        for i in range(tier_count):
            t = i / max(1, tier_count - 1)
            z = params.height * (0.24 + t * 0.68)
            radius = params.crown_radius * (1.0 - t * 0.72) * rng.uniform(0.86, 1.12)
            height = params.crown_height * rng.uniform(0.34, 0.55) * (1.0 - t * 0.25)
            offset = Vector((rng.uniform(-params.asymmetry, params.asymmetry) * (1.0 - t), rng.uniform(-params.asymmetry, params.asymmetry) * 0.6, z))
            tier = _cone_tier_mesh(f"PineTier_{i + 1:02d}", offset, radius, height, rng, collection, leaf_mats)
            _parent(tier, root)
            generated.append(tier)

    if params.show_scale_reference:
        generated.extend(_create_scale_reference(root, params, collection))

    bpy.context.scene[LAST_ROOT_PROP] = root.name
    _clear_selection()
    root.select_set(True)
    bpy.context.view_layer.objects.active = root
    bpy.context.view_layer.update()
    report = scene_report(root)
    root["poly_count"] = report.poly_count
    return asdict(report)


def _children_recursive(root):
    out = []
    stack = list(root.children)
    while stack:
        obj = stack.pop(0)
        out.append(obj)
        stack.extend(obj.children)
    return out


def scene_report(root=None):
    if root is None:
        root_name = bpy.context.scene.get(LAST_ROOT_PROP)
        root = bpy.data.objects.get(root_name) if root_name else bpy.context.active_object
    if root is None:
        raise ValueError("No generated tree root found")
    objects = [root] + _children_recursive(root)
    mesh_objects = [obj for obj in objects if obj.type == "MESH"]
    bbox_points = []
    for obj in mesh_objects:
        bbox_points.extend([obj.matrix_world @ Vector(corner) for corner in obj.bound_box])
    if bbox_points:
        bbox_min = [round(min(getattr(p, axis) for p in bbox_points), 4) for axis in ("x", "y", "z")]
        bbox_max = [round(max(getattr(p, axis) for p in bbox_points), 4) for axis in ("x", "y", "z")]
    else:
        bbox_min = [0.0, 0.0, 0.0]
        bbox_max = [0.0, 0.0, 0.0]
    params = {key: root.get(key) for key in ("seed", "tree_type", "placement_index")}
    return TreeReport(
        root_name=root.name,
        collection=COLLECTION_NAME,
        seed=int(root.get("seed", 0)),
        tree_type=str(root.get("tree_type", "")),
        parameters=params,
        objects=[obj.name for obj in objects],
        mesh_objects=len(mesh_objects),
        poly_count=sum(len(obj.data.polygons) for obj in mesh_objects),
        bbox_min=bbox_min,
        bbox_max=bbox_max,
    )


def clear_generated_trees():
    collection = bpy.data.collections.get(COLLECTION_NAME)
    if collection is None:
        return 0
    roots = [obj for obj in list(collection.objects) if obj.get(ROOT_PROP)]
    removed = 0
    for root in roots:
        for obj in reversed(_children_recursive(root)):
            if obj.name in bpy.data.objects:
                bpy.data.objects.remove(obj, do_unlink=True)
                removed += 1
        if root.name in bpy.data.objects:
            bpy.data.objects.remove(root, do_unlink=True)
            removed += 1
    for mat in list(bpy.data.materials):
        if mat.name.startswith("Tree_Bark_") or mat.name.startswith("Tree_Leaf_") or mat.name.startswith("Tree_Scale_"):
            bpy.data.materials.remove(mat)
    return removed


def export_tree_glb(filepath, root=None):
    if root is None:
        root_name = bpy.context.scene.get(LAST_ROOT_PROP)
        root = bpy.data.objects.get(root_name) if root_name else bpy.context.active_object
    if root is None:
        raise ValueError("No tree selected or generated")
    if not root.get(ROOT_PROP):
        parent = root
        while parent and not parent.get(ROOT_PROP):
            parent = parent.parent
        if parent is None:
            raise ValueError("Selected object is not part of a generated tree")
        root = parent
    out_path = Path(filepath)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    _clear_selection()
    for obj in [root] + _children_recursive(root):
        obj.select_set(True)
    bpy.context.view_layer.objects.active = root
    bpy.ops.export_scene.gltf(
        filepath=str(out_path),
        export_format="GLB",
        use_selection=True,
        export_yup=True,
        export_apply=True,
        export_materials="EXPORT",
        export_cameras=False,
        export_lights=False,
    )
    return str(out_path)


class LLMTreeSettings(bpy.types.PropertyGroup):
    seed: bpy.props.IntProperty(name="Seed", default=1234, min=0, update=lambda self, context: _schedule_auto_update(context))
    tree_type: bpy.props.EnumProperty(
        name="Type",
        default="broadleaf",
        items=(("broadleaf", "Broadleaf", "Chunky fantasy broadleaf"), ("pine", "Pine", "Stylized pine")),
        update=lambda self, context: _schedule_auto_update(context),
    )
    height: bpy.props.FloatProperty(name="Height (m)", default=4.2, min=0.8, max=12.0, step=10, update=lambda self, context: _schedule_auto_update(context))
    trunk_radius: bpy.props.FloatProperty(name="Trunk Radius (m)", default=0.24, min=0.04, max=1.2, step=1, update=lambda self, context: _schedule_auto_update(context))
    trunk_bend: bpy.props.FloatProperty(name="Trunk Bend (m)", default=0.35, min=0.0, max=2.0, step=1, update=lambda self, context: _schedule_auto_update(context))
    trunk_taper: bpy.props.FloatProperty(name="Trunk Taper", default=0.42, min=0.0, max=0.95, step=1, update=lambda self, context: _schedule_auto_update(context))
    branch_count: bpy.props.IntProperty(name="Branches", default=7, min=1, max=18, update=lambda self, context: _schedule_auto_update(context))
    branch_spread: bpy.props.FloatProperty(name="Branch Spread (m)", default=1.15, min=0.15, max=4.0, step=1, update=lambda self, context: _schedule_auto_update(context))
    clump_count: bpy.props.IntProperty(name="Clumps/Tiers", default=9, min=1, max=24, update=lambda self, context: _schedule_auto_update(context))
    crown_radius: bpy.props.FloatProperty(name="Crown Radius (m)", default=1.35, min=0.25, max=5.0, step=1, update=lambda self, context: _schedule_auto_update(context))
    crown_height: bpy.props.FloatProperty(name="Crown Height (m)", default=1.45, min=0.25, max=5.0, step=1, update=lambda self, context: _schedule_auto_update(context))
    asymmetry: bpy.props.FloatProperty(name="Asymmetry (m)", default=0.42, min=0.0, max=2.0, step=1, update=lambda self, context: _schedule_auto_update(context))
    exaggeration: bpy.props.FloatProperty(name="Exaggeration", default=0.65, min=0.0, max=1.5, step=1, update=lambda self, context: _schedule_auto_update(context))
    chunky_silhouette: bpy.props.FloatProperty(name="Chunky Silhouette", default=0.72, min=0.0, max=1.5, step=1, update=lambda self, context: _schedule_auto_update(context))
    color_variation: bpy.props.FloatProperty(name="Color Variation", default=0.30, min=0.0, max=1.0, step=1, update=lambda self, context: _schedule_auto_update(context))
    bark_hue: bpy.props.FloatProperty(name="Bark Hue", default=0.075, min=0.0, max=1.0, step=1, update=lambda self, context: _schedule_auto_update(context))
    foliage_hue: bpy.props.FloatProperty(name="Foliage Hue", default=0.28, min=0.0, max=1.0, step=1, update=lambda self, context: _schedule_auto_update(context))
    show_scale_reference: bpy.props.BoolProperty(name="1.80m Man", default=True, update=lambda self, context: _schedule_auto_update(context))
    scale_reference_distance: bpy.props.FloatProperty(name="Scale Distance (m)", default=0.8, min=0.1, max=5.0, step=1, update=lambda self, context: _schedule_auto_update(context))
    auto_update: bpy.props.BoolProperty(name="Auto Update", default=True)
    update_delay: bpy.props.FloatProperty(name="Delay (s)", default=0.0, min=0.0, max=2.0, step=1)
    export_path: bpy.props.StringProperty(name="Export Path", default="//assets/models/tree.glb", subtype="FILE_PATH")
    status: bpy.props.StringProperty(name="Status", default="Ready")
    last_seed: bpy.props.IntProperty(name="Last Seed", default=0)
    last_object_count: bpy.props.IntProperty(name="Objects", default=0)
    last_poly_count: bpy.props.IntProperty(name="Polygons", default=0)


def _params_from_settings(settings, seed=None):
    return TreeParams(
        seed=settings.seed if seed is None else seed,
        tree_type=settings.tree_type,
        height=settings.height,
        trunk_radius=settings.trunk_radius,
        trunk_bend=settings.trunk_bend,
        trunk_taper=settings.trunk_taper,
        branch_count=settings.branch_count,
        branch_spread=settings.branch_spread,
        clump_count=settings.clump_count,
        crown_radius=settings.crown_radius,
        crown_height=settings.crown_height,
        asymmetry=settings.asymmetry,
        exaggeration=settings.exaggeration,
        chunky_silhouette=settings.chunky_silhouette,
        color_variation=settings.color_variation,
        bark_hue=settings.bark_hue,
        foliage_hue=settings.foliage_hue,
        show_scale_reference=settings.show_scale_reference,
        scale_reference_distance=settings.scale_reference_distance,
    )


def _apply_report_to_settings(settings, report):
    settings.last_seed = int(report["seed"])
    settings.last_object_count = len(report["objects"])
    settings.last_poly_count = int(report["poly_count"])
    settings.status = f"Generated {report['root_name']}"


def _schedule_auto_update(context):
    if context is None or context.scene is None:
        return
    scene = context.scene
    if scene.get(SUPPRESS_UPDATE_PROP):
        return
    settings = getattr(scene, "llm_tree_settings", None)
    if settings is None or not settings.auto_update:
        return
    delay = max(0.0, float(settings.update_delay))
    if delay <= 1e-6:
        scene[SUPPRESS_UPDATE_PROP] = True
        try:
            report = create_tree(_params_from_settings(settings), replace_existing=True)
            _apply_report_to_settings(settings, report)
            settings.status = f"Live-updated {report['root_name']}"
        except Exception as exc:
            settings.status = f"Live update failed: {exc}"
        finally:
            scene[SUPPRESS_UPDATE_PROP] = False
        return

    scene[AUTO_UPDATE_DEADLINE_PROP] = time.monotonic() + delay
    if scene.get(AUTO_UPDATE_TIMER_PROP):
        return
    scene[AUTO_UPDATE_TIMER_PROP] = True
    bpy.app.timers.register(_auto_update_timer, first_interval=delay)


def _auto_update_timer():
    scene = bpy.context.scene
    if scene is None or not hasattr(scene, "llm_tree_settings"):
        return None
    deadline = float(scene.get(AUTO_UPDATE_DEADLINE_PROP, 0.0))
    remaining = deadline - time.monotonic()
    if remaining > 0.0:
        return min(remaining, 0.25)

    scene[AUTO_UPDATE_TIMER_PROP] = False
    settings = scene.llm_tree_settings
    if not settings.auto_update:
        return None
    scene[SUPPRESS_UPDATE_PROP] = True
    try:
        report = create_tree(_params_from_settings(settings), replace_existing=True)
        _apply_report_to_settings(settings, report)
        settings.status = f"Auto-updated {report['root_name']}"
    except Exception as exc:
        settings.status = f"Auto update failed: {exc}"
    finally:
        scene[SUPPRESS_UPDATE_PROP] = False
    return None


class LLM_TREE_OT_generate(bpy.types.Operator):
    bl_idname = "llm_tree.generate"
    bl_label = "Generate Tree"
    bl_options = {"REGISTER", "UNDO"}

    def execute(self, context):
        settings = context.scene.llm_tree_settings
        report = create_tree(_params_from_settings(settings), replace_existing=True)
        _apply_report_to_settings(settings, report)
        return {"FINISHED"}


class LLM_TREE_OT_clear(bpy.types.Operator):
    bl_idname = "llm_tree.clear"
    bl_label = "Clear Generated Trees"
    bl_options = {"REGISTER", "UNDO"}

    def execute(self, context):
        removed = clear_generated_trees()
        settings = context.scene.llm_tree_settings
        settings.status = f"Removed {removed} objects"
        settings.last_object_count = 0
        settings.last_poly_count = 0
        return {"FINISHED"}


class LLM_TREE_OT_export_glb(bpy.types.Operator):
    bl_idname = "llm_tree.export_glb"
    bl_label = "Export Selected GLB"

    def execute(self, context):
        settings = context.scene.llm_tree_settings
        path = bpy.path.abspath(settings.export_path)
        try:
            exported = export_tree_glb(path)
        except Exception as exc:
            settings.status = str(exc)
            self.report({"ERROR"}, str(exc))
            return {"CANCELLED"}
        settings.status = f"Exported {exported}"
        return {"FINISHED"}


class LLM_TREE_PT_panel(bpy.types.Panel):
    bl_label = "LLM Trees"
    bl_idname = "LLM_TREE_PT_panel"
    bl_space_type = "VIEW_3D"
    bl_region_type = "UI"
    bl_category = "LLM Trees"

    def draw(self, context):
        layout = self.layout
        settings = context.scene.llm_tree_settings

        layout.prop(settings, "seed")
        layout.prop(settings, "tree_type")
        layout.separator()
        col = layout.column(align=True)
        for prop in ("height", "trunk_radius", "trunk_bend", "trunk_taper", "branch_count", "branch_spread"):
            col.prop(settings, prop)
        layout.separator()
        col = layout.column(align=True)
        for prop in ("clump_count", "crown_radius", "crown_height", "asymmetry", "exaggeration", "chunky_silhouette"):
            col.prop(settings, prop)
        layout.separator()
        col = layout.column(align=True)
        for prop in ("color_variation", "bark_hue", "foliage_hue"):
            col.prop(settings, prop)
        layout.separator()
        row = layout.row(align=True)
        row.prop(settings, "show_scale_reference")
        row.prop(settings, "scale_reference_distance")
        layout.separator()
        row = layout.row(align=True)
        row.prop(settings, "auto_update")
        row.prop(settings, "update_delay")
        layout.operator("llm_tree.generate", icon="OUTLINER_OB_FORCE_FIELD")
        layout.operator("llm_tree.clear", icon="TRASH")
        layout.separator()
        layout.prop(settings, "export_path")
        layout.operator("llm_tree.export_glb", icon="EXPORT")
        layout.separator()
        layout.label(text=f"Status: {settings.status}")
        layout.label(text=f"Last seed: {settings.last_seed}")
        layout.label(text=f"Objects: {settings.last_object_count}  Polys: {settings.last_poly_count}")


CLASSES = (
    LLMTreeSettings,
    LLM_TREE_OT_generate,
    LLM_TREE_OT_clear,
    LLM_TREE_OT_export_glb,
    LLM_TREE_PT_panel,
)


def register():
    for cls in CLASSES:
        bpy.utils.register_class(cls)
    bpy.types.Scene.llm_tree_settings = bpy.props.PointerProperty(type=LLMTreeSettings)


def unregister():
    if hasattr(bpy.types.Scene, "llm_tree_settings"):
        del bpy.types.Scene.llm_tree_settings
    for cls in reversed(CLASSES):
        bpy.utils.unregister_class(cls)


def _parse_args():
    parser = argparse.ArgumentParser(description="Generate deterministic stylized trees in Blender.")
    parser.add_argument("--seed", type=int, default=1234)
    parser.add_argument("--tree-type", choices=("broadleaf", "pine"), default="broadleaf")
    parser.add_argument("--height", type=float, default=4.2)
    parser.add_argument("--trunk-radius", type=float, default=0.24)
    parser.add_argument("--trunk-bend", type=float, default=0.35)
    parser.add_argument("--trunk-taper", type=float, default=0.42)
    parser.add_argument("--branch-count", type=int, default=7)
    parser.add_argument("--branch-spread", type=float, default=1.15)
    parser.add_argument("--clump-count", type=int, default=9)
    parser.add_argument("--crown-radius", type=float, default=1.35)
    parser.add_argument("--crown-height", type=float, default=1.45)
    parser.add_argument("--asymmetry", type=float, default=0.42)
    parser.add_argument("--exaggeration", type=float, default=0.65)
    parser.add_argument("--chunky-silhouette", type=float, default=0.72)
    parser.add_argument("--color-variation", type=float, default=0.30)
    parser.add_argument("--bark-hue", type=float, default=0.075)
    parser.add_argument("--foliage-hue", type=float, default=0.28)
    parser.add_argument("--hide-scale-reference", action="store_true")
    parser.add_argument("--scale-reference-distance", type=float, default=0.8)
    parser.add_argument("--clear", action="store_true")
    parser.add_argument("--validate", action="store_true")
    parser.add_argument("--report-json", type=str, default=None)
    parser.add_argument("--export-glb", type=str, default=None)
    return parser.parse_args(_parse_blender_argv())


def _main():
    args = _parse_args()
    if args.clear:
        clear_generated_trees()
    params = TreeParams(
        seed=args.seed,
        tree_type=args.tree_type,
        height=args.height,
        trunk_radius=args.trunk_radius,
        trunk_bend=args.trunk_bend,
        trunk_taper=args.trunk_taper,
        branch_count=args.branch_count,
        branch_spread=args.branch_spread,
        clump_count=args.clump_count,
        crown_radius=args.crown_radius,
        crown_height=args.crown_height,
        asymmetry=args.asymmetry,
        exaggeration=args.exaggeration,
        chunky_silhouette=args.chunky_silhouette,
        color_variation=args.color_variation,
        bark_hue=args.bark_hue,
        foliage_hue=args.foliage_hue,
        show_scale_reference=not args.hide_scale_reference,
        scale_reference_distance=args.scale_reference_distance,
    )
    report = create_tree(params)
    if args.report_json:
        out_path = Path(args.report_json)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    if args.export_glb:
        export_tree_glb(args.export_glb, bpy.data.objects[report["root_name"]])
    if args.validate:
        if report["mesh_objects"] < 3 or report["poly_count"] < 40:
            print("VALIDATION: FAIL")
            raise SystemExit(2)
        print("VALIDATION: PASS")


if __name__ == "__main__":
    _main()
