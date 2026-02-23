import argparse
import math
import sys
from pathlib import Path

import bpy
from mathutils import Vector

SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from blender_utils import (  # noqa: E402
    add_common_output_args,
    clear_collection,
    create_cube,
    ensure_camera_and_light,
    ensure_object_mode,
    get_or_create_collection,
    maybe_render,
    maybe_write_report,
    move_to_collection,
    parse_blender_argv,
)

COLLECTION_NAME = "LLM_Table"


def _set_flat_shading(obj):
    for poly in obj.data.polygons:
        poly.use_smooth = False


def _remove_startup_cube():
    cube = bpy.data.objects.get("Cube")
    if cube is None or cube.type != "MESH":
        return
    # Blender startup cube signature: 8 verts at origin, 2x2x2 dimensions.
    if len(cube.data.vertices) != 8:
        return
    if cube.location.length > 1e-6:
        return
    dims = cube.dimensions
    if not (abs(dims.x - 2.0) < 1e-5 and abs(dims.y - 2.0) < 1e-5 and abs(dims.z - 2.0) < 1e-5):
        return
    bpy.data.objects.remove(cube, do_unlink=True)


def _clear_selection():
    for obj in bpy.context.selected_objects:
        obj.select_set(False)


def _apply_scale(obj):
    _clear_selection()
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)


def _mesh_dims_local(obj):
    verts = obj.data.vertices
    xs = [v.co.x for v in verts]
    ys = [v.co.y for v in verts]
    zs = [v.co.z for v in verts]
    return (max(xs) - min(xs), max(ys) - min(ys), max(zs) - min(zs))


def _taper_mesh_z(obj, bottom_scale=1.0, top_scale=1.0):
    mesh = obj.data
    zs = [v.co.z for v in mesh.vertices]
    z_min = min(zs)
    z_max = max(zs)
    span = max(z_max - z_min, 1e-8)

    for v in mesh.vertices:
        t = (v.co.z - z_min) / span
        s = bottom_scale + (top_scale - bottom_scale) * t
        v.co.x *= s
        v.co.y *= s

    mesh.update()


def _warp_top_surface(obj, strength):
    if abs(strength) <= 1e-9:
        return

    mesh = obj.data
    z_max = max(v.co.z for v in mesh.vertices)
    for v in mesh.vertices:
        if abs(v.co.z - z_max) <= 1e-6:
            # Alternate opposite corners up/down for a subtle non-orthogonal silhouette.
            sign = 1.0 if (v.co.x * v.co.y) >= 0 else -1.0
            v.co.z += sign * strength

    mesh.update()


def _leg_splay_rotation(x, y, splay_deg):
    angle = math.radians(splay_deg)
    rot_x = math.copysign(angle, y)
    rot_y = math.copysign(angle, x)
    return (rot_x, rot_y, 0.0)


def _bbox_world_z_extents(obj):
    points = [obj.matrix_world @ Vector(corner) for corner in obj.bound_box]
    zs = [p.z for p in points]
    return min(zs), max(zs)


def _parent_keep_world(child, parent):
    _clear_selection()
    parent.select_set(True)
    child.select_set(True)
    bpy.context.view_layer.objects.active = parent
    bpy.ops.object.parent_set(type="OBJECT", keep_transform=True)


def _compute_style_metrics(top_width, top_depth, leg_thickness, inset, top_taper, leg_taper, leg_splay_deg, top_warp):
    reference_size = min(top_width, top_depth)
    leg_ratio = leg_thickness / reference_size
    inset_ratio = inset / reference_size
    non_boxy_features = sum(
        [
            leg_splay_deg >= 3.0,
            top_taper <= 0.95,
            leg_taper <= 0.9,
            abs(top_warp) > 0.0,
        ]
    )
    return {
        "top_width": round(top_width, 4),
        "top_depth": round(top_depth, 4),
        "top_aspect_ratio": round(top_width / max(top_depth, 1e-8), 4),
        "leg_thickness_ratio": round(leg_ratio, 4),
        "inset_ratio": round(inset_ratio, 4),
        "non_boxy_feature_count": int(non_boxy_features),
        "non_boxy_score": round(non_boxy_features / 4.0, 4),
    }


def _export_glb(filepath, objects):
    out_path = Path(filepath)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    _clear_selection()
    for obj in objects:
        obj.select_set(True)

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


def create_low_poly_table(
    top_width,
    top_depth,
    top_thickness,
    table_height,
    leg_thickness,
    inset,
    top_taper=0.9,
    leg_taper=0.82,
    leg_splay_deg=5.0,
    top_warp=0.008,
    bevel=False,
    bevel_width=0.01,
    bevel_segments=1,
):
    if table_height <= top_thickness:
        raise ValueError("table_height must be greater than top_thickness")
    if inset < 0:
        raise ValueError("inset must be >= 0")
    if not (0.6 <= top_taper <= 1.0):
        raise ValueError("top_taper must be in range [0.6, 1.0]")
    if not (0.6 <= leg_taper <= 1.0):
        raise ValueError("leg_taper must be in range [0.6, 1.0]")
    if not (0.0 <= leg_splay_deg <= 20.0):
        raise ValueError("leg_splay_deg must be in range [0.0, 20.0]")
    if abs(top_warp) > top_thickness * 0.45:
        raise ValueError("top_warp is too large for the given top_thickness")

    leg_height = table_height - top_thickness
    reference_size = min(top_width, top_depth)
    max_inset = (reference_size - leg_thickness) / 2.0
    if max_inset <= 0:
        raise ValueError("top_width/top_depth must be greater than leg_thickness")
    if inset > max_inset:
        raise ValueError(
            f"inset is too large for given dimensions; max inset is {max_inset:.4f}"
        )

    _remove_startup_cube()
    # Keep projected table height stable even when legs are splayed in X and Y.
    splay_rad = math.radians(leg_splay_deg)
    projected_factor = max(math.cos(splay_rad) * math.cos(splay_rad), 1e-5)
    leg_mesh_height = leg_height / projected_factor

    collection = get_or_create_collection(COLLECTION_NAME)
    clear_collection(collection)

    top_z = table_height - top_thickness / 2.0
    table_top = create_cube(
        name="TableTop",
        location=(0.0, 0.0, top_z),
        dimensions=(top_width, top_depth, top_thickness),
        collection=collection,
    )
    _apply_scale(table_top)
    _taper_mesh_z(table_top, bottom_scale=top_taper, top_scale=1.0)
    _warp_top_surface(table_top, top_warp)
    _set_flat_shading(table_top)

    template_leg = create_cube(
        name="TableLeg_Template",
        location=(0.0, 0.0, leg_height / 2.0),
        dimensions=(leg_thickness, leg_thickness, leg_mesh_height),
        collection=collection,
    )
    _apply_scale(template_leg)
    _taper_mesh_z(template_leg, bottom_scale=1.0, top_scale=leg_taper)
    _set_flat_shading(template_leg)

    offset_x = top_width * 0.5 - inset - leg_thickness * 0.5
    offset_y = top_depth * 0.5 - inset - leg_thickness * 0.5
    leg_z = leg_height / 2.0
    leg_positions = [
        (offset_x, offset_y, leg_z),
        (offset_x, -offset_y, leg_z),
        (-offset_x, offset_y, leg_z),
        (-offset_x, -offset_y, leg_z),
    ]

    legs = []
    for i, pos in enumerate(leg_positions, start=1):
        leg = template_leg.copy()
        leg.data = template_leg.data.copy()
        leg.name = f"TableLeg.{i:03d}"
        move_to_collection(leg, collection)
        leg.location = Vector(pos)
        leg.rotation_euler = _leg_splay_rotation(pos[0], pos[1], leg_splay_deg)
        _set_flat_shading(leg)
        _parent_keep_world(leg, table_top)

        if bevel:
            mod = leg.modifiers.new(name="Bevel", type="BEVEL")
            mod.width = bevel_width
            mod.segments = max(1, int(bevel_segments))
            mod.limit_method = "ANGLE"
            mod.angle_limit = 0.785398
        legs.append(leg)

    bpy.data.objects.remove(template_leg, do_unlink=True)

    if bevel:
        mod = table_top.modifiers.new(name="Bevel", type="BEVEL")
        mod.width = bevel_width
        mod.segments = max(1, int(bevel_segments))
        mod.limit_method = "ANGLE"
        mod.angle_limit = 0.785398

    bpy.context.view_layer.update()
    style_metrics = _compute_style_metrics(
        top_width=top_width,
        top_depth=top_depth,
        leg_thickness=leg_thickness,
        inset=inset,
        top_taper=top_taper,
        leg_taper=leg_taper,
        leg_splay_deg=leg_splay_deg,
        top_warp=top_warp,
    )
    return table_top, legs, leg_height, leg_mesh_height, style_metrics


def scene_report(table_top, legs, leg_height, leg_mesh_height, style_metrics):
    def rounded(values):
        return [round(float(v), 4) for v in values]

    return {
        "collection": COLLECTION_NAME,
        "objects": [table_top.name] + [leg.name for leg in legs],
        "table_top": {
            "location": rounded(table_top.location),
            "dimensions": rounded(table_top.dimensions),
        },
        "leg_height": round(float(leg_height), 4),
        "leg_mesh_height": round(float(leg_mesh_height), 4),
        "style_metrics": style_metrics,
        "legs": [
            {
                "name": leg.name,
                "location": rounded(leg.location),
                "world_location": rounded(leg.matrix_world.translation),
                "dimensions": rounded(leg.dimensions),
                "rotation_deg": rounded(
                    [math.degrees(leg.rotation_euler.x), math.degrees(leg.rotation_euler.y), math.degrees(leg.rotation_euler.z)]
                ),
                "parent": leg.parent.name if leg.parent else None,
            }
            for leg in legs
        ],
        "poly_total": int(
            sum(
                len(obj.data.polygons)
                for obj in [table_top, *legs]
                if obj.type == "MESH"
            )
        ),
    }


def validate_table(
    table_top,
    legs,
    top_width,
    top_depth,
    top_thickness,
    table_height,
    leg_thickness,
    inset,
    top_taper,
    leg_taper,
    leg_splay_deg,
    top_warp,
    leg_mesh_height,
):
    eps = 1e-3
    issues = []

    def close(a, b, tol=eps):
        return abs(float(a) - float(b)) <= tol

    if len(legs) != 4:
        issues.append(f"Expected 4 legs, got {len(legs)}")

    leg_height = table_height - top_thickness
    top_dims_local = _mesh_dims_local(table_top)
    expected_top_z_dim = top_thickness + max(0.0, top_warp)
    if not close(top_dims_local[0], top_width) or not close(top_dims_local[1], top_depth):
        issues.append(
            f"TableTop local XY mismatch: got {top_dims_local[:2]}, expected {(top_width, top_depth)}"
        )
    if not close(top_dims_local[2], expected_top_z_dim):
        issues.append(
            f"TableTop local Z mismatch: got {top_dims_local[2]}, expected {expected_top_z_dim}"
        )

    top_z_expected = table_height - top_thickness / 2.0
    if not close(table_top.location.z, top_z_expected):
        issues.append(
            f"TableTop z mismatch: got {table_top.location.z}, expected {top_z_expected}"
        )

    expected_offset_x = top_width * 0.5 - inset - leg_thickness * 0.5
    expected_offset_y = top_depth * 0.5 - inset - leg_thickness * 0.5
    expected_xy = {
        (expected_offset_x, expected_offset_y),
        (expected_offset_x, -expected_offset_y),
        (-expected_offset_x, expected_offset_y),
        (-expected_offset_x, -expected_offset_y),
    }

    expected_top_touch = table_height - top_thickness
    splay_angle = math.radians(abs(leg_splay_deg))
    corner_z_offset = leg_thickness * math.sin(splay_angle)
    z_touch_tolerance = max(5e-3, corner_z_offset * 1.2)
    actual_xy = set()
    for leg in legs:
        dim = _mesh_dims_local(leg)
        if not close(dim[0], leg_thickness) or not close(dim[1], leg_thickness):
            issues.append(
                f"{leg.name} local XY mismatch: got {dim[:2]}, expected {(leg_thickness, leg_thickness)}"
            )
        if not close(dim[2], leg_mesh_height):
            issues.append(
                f"{leg.name} local Z mismatch: got {dim[2]}, expected {leg_mesh_height}"
            )

        leg_min_z, leg_max_z = _bbox_world_z_extents(leg)
        if not close(leg_min_z, 0.0, tol=z_touch_tolerance):
            issues.append(f"{leg.name} does not touch ground: min_z={leg_min_z}")
        if not close(leg_max_z, expected_top_touch, tol=z_touch_tolerance):
            issues.append(
                f"{leg.name} does not touch tabletop underside: max_z={leg_max_z}, expected={expected_top_touch}"
            )
        if leg.parent != table_top:
            issues.append(f"{leg.name} is not parented to TableTop")

        world_loc = leg.matrix_world.translation
        actual_xy.add((round(float(world_loc.x), 4), round(float(world_loc.y), 4)))

    expected_xy_rounded = {(round(x, 4), round(y, 4)) for x, y in expected_xy}
    if actual_xy != expected_xy_rounded:
        issues.append(
            f"Leg corner positions mismatch: got {sorted(actual_xy)}, expected {sorted(expected_xy_rounded)}"
        )

    # Style heuristics (AGENT.md): non-boxy + readability over realism.
    reference_size = min(top_width, top_depth)
    leg_ratio = leg_thickness / reference_size
    inset_ratio = inset / reference_size
    if not (0.06 <= leg_ratio <= 0.16):
        issues.append(
            f"Readability ratio leg_thickness/min(top_width,top_depth) out of range: {leg_ratio:.4f} (expected 0.06..0.16)"
        )
    if not (0.04 <= inset_ratio <= 0.20):
        issues.append(
            f"Readability ratio inset/min(top_width,top_depth) out of range: {inset_ratio:.4f} (expected 0.04..0.20)"
        )
    if leg_splay_deg < 3.0:
        issues.append(
            f"Non-boxy rule failed: leg_splay_deg={leg_splay_deg} (expected >= 3.0)"
        )
    if top_taper > 0.95 and leg_taper > 0.9 and abs(top_warp) <= 1e-9:
        issues.append(
            "Non-boxy rule failed: no taper/warp signal (top_taper too high, leg_taper too high, top_warp=0)"
        )

    return issues


def parse_args():
    parser = argparse.ArgumentParser(
        description="Generate a stylized low-poly table with non-boxy readability heuristics."
    )
    parser.add_argument("--top-width", type=float, default=1.2)
    parser.add_argument("--top-depth", type=float, default=1.2)
    parser.add_argument(
        "--top-size",
        type=float,
        default=None,
        help="Legacy alias to set both top width and top depth.",
    )
    parser.add_argument("--top-thickness", type=float, default=0.08)
    parser.add_argument("--table-height", type=float, default=0.75)
    parser.add_argument("--leg-thickness", type=float, default=0.10)
    parser.add_argument("--inset", type=float, default=0.08)
    parser.add_argument("--top-taper", type=float, default=0.90)
    parser.add_argument("--leg-taper", type=float, default=0.82)
    parser.add_argument("--leg-splay-deg", type=float, default=5.0)
    parser.add_argument("--top-warp", type=float, default=0.008)
    parser.add_argument("--bevel", action="store_true")
    parser.add_argument("--bevel-width", type=float, default=0.01)
    parser.add_argument("--bevel-segments", type=int, default=1)
    parser.add_argument("--export-glb", type=str, default=None)
    add_common_output_args(parser)
    return parser.parse_args(parse_blender_argv())


def main():
    ensure_object_mode()
    args = parse_args()
    top_width = args.top_size if args.top_size is not None else args.top_width
    top_depth = args.top_size if args.top_size is not None else args.top_depth
    table_top, legs, leg_height, leg_mesh_height, style_metrics = create_low_poly_table(
        top_width=top_width,
        top_depth=top_depth,
        top_thickness=args.top_thickness,
        table_height=args.table_height,
        leg_thickness=args.leg_thickness,
        inset=args.inset,
        top_taper=args.top_taper,
        leg_taper=args.leg_taper,
        leg_splay_deg=args.leg_splay_deg,
        top_warp=args.top_warp,
        bevel=args.bevel,
        bevel_width=args.bevel_width,
        bevel_segments=args.bevel_segments,
    )

    report = scene_report(
        table_top=table_top,
        legs=legs,
        leg_height=leg_height,
        leg_mesh_height=leg_mesh_height,
        style_metrics=style_metrics,
    )
    maybe_write_report(args.report_json, report)
    if args.render:
        ensure_camera_and_light(target=(0.0, 0.0, args.table_height * 0.5))
        maybe_render(args.render)
    if args.export_glb:
        _export_glb(args.export_glb, [table_top, *legs])

    if args.validate:
        issues = validate_table(
            table_top=table_top,
            legs=legs,
            top_width=top_width,
            top_depth=top_depth,
            top_thickness=args.top_thickness,
            table_height=args.table_height,
            leg_thickness=args.leg_thickness,
            inset=args.inset,
            top_taper=args.top_taper,
            leg_taper=args.leg_taper,
            leg_splay_deg=args.leg_splay_deg,
            top_warp=args.top_warp,
            leg_mesh_height=leg_mesh_height,
        )
        if issues:
            print("VALIDATION: FAIL")
            for issue in issues:
                print(f"- {issue}")
            raise SystemExit(2)
        print("VALIDATION: PASS")


main()
