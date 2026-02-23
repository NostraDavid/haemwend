use crate::blender_model_editor::state::EditorState;
use crate::blender_model_editor::{GRID_EXTENT_METERS, GRID_MAJOR_STEP_METERS};
use bevy::asset::RenderAssetUsages;
use bevy::camera::ClearColorConfig;
use bevy::camera::visibility::RenderLayers;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy_egui::PrimaryEguiContext;

#[derive(Resource)]
pub struct PreviewScene {
    pub top_mesh: Handle<Mesh>,
    pub leg_mesh: Handle<Mesh>,
    pub top_entity: Entity,
    pub leg_entities: [Entity; 4],
}

#[derive(Component)]
pub struct PreviewCamera;

#[derive(Debug, Clone, Copy)]
pub struct TablePreviewParams {
    pub top_width: f32,
    pub top_depth: f32,
    pub top_thickness: f32,
    pub table_height: f32,
    pub leg_thickness: f32,
    pub inset: f32,
    pub top_taper: f32,
    pub leg_taper: f32,
    pub leg_splay_deg: f32,
    pub top_warp: f32,
    pub leg_mesh_height: f32,
}

impl TablePreviewParams {
    pub fn from_state(state: &EditorState) -> Result<Self, String> {
        let top_width = state.get_f32("top-width", 1.2);
        let top_depth = state.get_f32("top-depth", 1.2);
        let top_thickness = state.get_f32("top-thickness", 0.08);
        let table_height = state.get_f32("table-height", 0.75);
        let leg_thickness = state.get_f32("leg-thickness", 0.10);
        let inset = state.get_f32("inset", 0.08);
        let top_taper = state.get_f32("top-taper", 0.90);
        let leg_taper = state.get_f32("leg-taper", 0.82);
        let leg_splay_deg = state.get_f32("leg-splay-deg", 5.0);
        let top_warp = state.get_f32("top-warp", 0.008);

        if top_width <= 0.0 || top_depth <= 0.0 {
            return Err("top-width and top-depth must be > 0".to_string());
        }
        if top_thickness <= 0.0 {
            return Err("top-thickness must be > 0".to_string());
        }
        if table_height <= top_thickness {
            return Err("table-height must be greater than top-thickness".to_string());
        }
        if leg_thickness <= 0.0 {
            return Err("leg-thickness must be > 0".to_string());
        }
        if inset < 0.0 {
            return Err("inset must be >= 0".to_string());
        }
        if !(0.6..=1.0).contains(&top_taper) {
            return Err("top-taper must be in range [0.6, 1.0]".to_string());
        }
        if !(0.6..=1.0).contains(&leg_taper) {
            return Err("leg-taper must be in range [0.6, 1.0]".to_string());
        }
        if !(0.0..=20.0).contains(&leg_splay_deg) {
            return Err("leg-splay-deg must be in range [0.0, 20.0]".to_string());
        }
        if top_warp.abs() > top_thickness * 0.45 {
            return Err("top-warp is too large for current top-thickness".to_string());
        }

        let reference_size = top_width.min(top_depth);
        let max_inset = (reference_size - leg_thickness) * 0.5;
        if max_inset <= 0.0 {
            return Err("top-width/top-depth must be greater than leg-thickness".to_string());
        }
        if inset > max_inset {
            return Err(format!(
                "inset is too large for dimensions; max inset is {max_inset:.4}"
            ));
        }

        let leg_height = table_height - top_thickness;
        let splay_rad = leg_splay_deg.to_radians();
        let projected_factor = (splay_rad.cos() * splay_rad.cos()).max(1e-5);
        let leg_mesh_height = leg_height / projected_factor;

        Ok(Self {
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
        })
    }
}

pub fn setup_preview_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((Camera3d::default(), Transform::default(), PreviewCamera));
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(31),
        PrimaryEguiContext,
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.84),
            shadows_enabled: true,
            illuminance: 14_000.0,
            ..default()
        },
        Transform::from_xyz(4.0, -5.0, 7.0).looking_at(Vec3::new(0.0, 0.0, 0.45), Vec3::Z),
    ));

    let top_mesh = meshes.add(build_tapered_box_mesh(1.0, 1.0, 0.1, 0.9, 1.0, 0.0));
    let leg_mesh = meshes.add(build_tapered_box_mesh(0.1, 0.1, 0.7, 1.0, 0.82, 0.0));

    let top_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.62, 0.42, 0.24),
        perceptual_roughness: 0.93,
        ..default()
    });
    let leg_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.51, 0.33, 0.19),
        perceptual_roughness: 0.95,
        ..default()
    });

    let top_entity = commands
        .spawn((
            Mesh3d(top_mesh.clone()),
            MeshMaterial3d(top_mat),
            Transform::from_xyz(0.0, 0.0, 0.7),
        ))
        .id();

    let leg_entities = [
        commands
            .spawn((
                Mesh3d(leg_mesh.clone()),
                MeshMaterial3d(leg_mat.clone()),
                Transform::default(),
            ))
            .id(),
        commands
            .spawn((
                Mesh3d(leg_mesh.clone()),
                MeshMaterial3d(leg_mat.clone()),
                Transform::default(),
            ))
            .id(),
        commands
            .spawn((
                Mesh3d(leg_mesh.clone()),
                MeshMaterial3d(leg_mat.clone()),
                Transform::default(),
            ))
            .id(),
        commands
            .spawn((
                Mesh3d(leg_mesh.clone()),
                MeshMaterial3d(leg_mat),
                Transform::default(),
            ))
            .id(),
    ];

    commands.insert_resource(PreviewScene {
        top_mesh,
        leg_mesh,
        top_entity,
        leg_entities,
    });
}

pub fn queue_initial_preview(mut state: ResMut<EditorState>) {
    state.dirty = true;
    state.request_center_view = true;
}

pub fn apply_live_preview(
    mut state: ResMut<EditorState>,
    preview: Res<PreviewScene>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut transforms: Query<&mut Transform>,
) {
    if !state.dirty {
        return;
    }

    if state.current_model().id != "table" {
        state.status = format!(
            "Live preview unavailable for model '{}' (add Rust preview generator)",
            state.current_model().id
        );
        state.dirty = false;
        return;
    }

    let params = match TablePreviewParams::from_state(&state) {
        Ok(params) => params,
        Err(err) => {
            state.status = format!("Preview parameter error: {err}");
            state.dirty = false;
            return;
        }
    };

    if let Some(mesh) = meshes.get_mut(&preview.top_mesh) {
        *mesh = build_tapered_box_mesh(
            params.top_width,
            params.top_depth,
            params.top_thickness,
            params.top_taper,
            1.0,
            params.top_warp,
        );
    }

    if let Some(mesh) = meshes.get_mut(&preview.leg_mesh) {
        *mesh = build_tapered_box_mesh(
            params.leg_thickness,
            params.leg_thickness,
            params.leg_mesh_height,
            1.0,
            params.leg_taper,
            0.0,
        );
    }

    if let Ok(mut top_transform) = transforms.get_mut(preview.top_entity) {
        *top_transform =
            Transform::from_xyz(0.0, 0.0, params.table_height - params.top_thickness * 0.5);
    }

    let leg_height = params.table_height - params.top_thickness;
    let offset_x = params.top_width * 0.5 - params.inset - params.leg_thickness * 0.5;
    let offset_y = params.top_depth * 0.5 - params.inset - params.leg_thickness * 0.5;
    let leg_z = leg_height * 0.5;
    let splay_rad = params.leg_splay_deg.to_radians();

    let leg_positions = [
        Vec3::new(offset_x, offset_y, leg_z),
        Vec3::new(offset_x, -offset_y, leg_z),
        Vec3::new(-offset_x, offset_y, leg_z),
        Vec3::new(-offset_x, -offset_y, leg_z),
    ];

    for (entity, pos) in preview.leg_entities.iter().zip(leg_positions) {
        let rot_x = splay_rad.copysign(pos.y);
        let rot_y = splay_rad.copysign(pos.x);
        let rotation = Quat::from_euler(EulerRot::XYZ, rot_x, rot_y, 0.0);

        if let Ok(mut transform) = transforms.get_mut(*entity) {
            *transform = Transform::from_translation(pos).with_rotation(rotation);
        }
    }

    state.status = "Live preview updated".to_string();
    state.dirty = false;
}

pub fn frame_camera_target_distance(state: &EditorState) -> (Vec3, f32) {
    if state.current_model().id == "table" {
        if let Ok(params) = TablePreviewParams::from_state(state) {
            let radius = params
                .top_width
                .max(params.top_depth)
                .max(params.table_height)
                * 0.95;
            let target = Vec3::new(0.0, 0.0, params.table_height * 0.42);
            let distance = (radius * 3.4).clamp(1.8, 25.0);
            return (target, distance);
        }
    }

    (Vec3::new(0.0, 0.0, 0.5), 4.0)
}

pub fn draw_grid_system(mut gizmos: Gizmos, state: Res<EditorState>) {
    if !state.show_grid {
        return;
    }

    let extent = GRID_EXTENT_METERS as f32;
    let z = 0.001;

    for i in -GRID_EXTENT_METERS..=GRID_EXTENT_METERS {
        let f = i as f32;
        let is_major = i % GRID_MAJOR_STEP_METERS == 0;
        let color = if is_major {
            Color::srgba(0.55, 0.55, 0.55, 0.55)
        } else {
            Color::srgba(0.32, 0.32, 0.32, 0.35)
        };

        gizmos.line(Vec3::new(-extent, f, z), Vec3::new(extent, f, z), color);
        gizmos.line(Vec3::new(f, -extent, z), Vec3::new(f, extent, z), color);
    }

    gizmos.line(
        Vec3::new(-extent, 0.0, z + 0.0005),
        Vec3::new(extent, 0.0, z + 0.0005),
        Color::srgb(0.85, 0.25, 0.25),
    );
    gizmos.line(
        Vec3::new(0.0, -extent, z + 0.0005),
        Vec3::new(0.0, extent, z + 0.0005),
        Color::srgb(0.25, 0.85, 0.25),
    );
}

pub fn grid_info_text() -> String {
    let full_size = GRID_EXTENT_METERS * 2;
    format!(
        "Grid scale: 1 vak = 1 meter, dikke lijn elke {}m, bereik: -{}..+{}m ({}x{}m).",
        GRID_MAJOR_STEP_METERS, GRID_EXTENT_METERS, GRID_EXTENT_METERS, full_size, full_size
    )
}

fn build_tapered_box_mesh(
    width: f32,
    depth: f32,
    height: f32,
    bottom_scale: f32,
    top_scale: f32,
    top_warp: f32,
) -> Mesh {
    let half_z = height * 0.5;
    let bottom_x = width * 0.5 * bottom_scale;
    let bottom_y = depth * 0.5 * bottom_scale;
    let top_x = width * 0.5 * top_scale;
    let top_y = depth * 0.5 * top_scale;

    let warp = |x_sign: f32, y_sign: f32| {
        if x_sign * y_sign >= 0.0 {
            top_warp
        } else {
            -top_warp
        }
    };

    let b_lb = Vec3::new(-bottom_x, -bottom_y, -half_z);
    let b_rb = Vec3::new(bottom_x, -bottom_y, -half_z);
    let b_rf = Vec3::new(bottom_x, bottom_y, -half_z);
    let b_lf = Vec3::new(-bottom_x, bottom_y, -half_z);

    let t_lb = Vec3::new(-top_x, -top_y, half_z + warp(-1.0, -1.0));
    let t_rb = Vec3::new(top_x, -top_y, half_z + warp(1.0, -1.0));
    let t_rf = Vec3::new(top_x, top_y, half_z + warp(1.0, 1.0));
    let t_lf = Vec3::new(-top_x, top_y, half_z + warp(-1.0, 1.0));

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(24);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(24);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(24);
    let mut indices: Vec<u32> = Vec::with_capacity(36);

    add_quad(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        [b_lf, b_rf, t_rf, t_lf],
        Vec3::Y,
    );
    add_quad(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        [b_rb, b_lb, t_lb, t_rb],
        -Vec3::Y,
    );
    add_quad(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        [b_rf, b_rb, t_rb, t_rf],
        Vec3::X,
    );
    add_quad(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        [b_lb, b_lf, t_lf, t_lb],
        -Vec3::X,
    );
    add_quad(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        [t_lb, t_rb, t_rf, t_lf],
        Vec3::Z,
    );
    add_quad(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        [b_lf, b_rf, b_rb, b_lb],
        -Vec3::Z,
    );

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn add_quad(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    mut corners: [Vec3; 4],
    expected_normal: Vec3,
) {
    let mut normal = (corners[1] - corners[0]).cross(corners[2] - corners[0]);
    if normal.dot(expected_normal) < 0.0 {
        corners = [corners[0], corners[3], corners[2], corners[1]];
        normal = (corners[1] - corners[0]).cross(corners[2] - corners[0]);
    }
    let normal = normal.normalize_or_zero();

    let base = positions.len() as u32;
    for corner in corners {
        positions.push([corner.x, corner.y, corner.z]);
        normals.push([normal.x, normal.y, normal.z]);
    }
    uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}
