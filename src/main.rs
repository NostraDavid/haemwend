use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "haemwend".into(),
                resolution: WindowResolution::new(1920, 1080),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin {
                wait_duration: Duration::from_secs(2),
                ..default()
            },
        ))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.6, 0.65, 0.7),
            brightness: 250.0,
            affects_lightmapped_meshes: true,
        })
        .add_systems(Startup, setup_world)
        .add_systems(Update, (player_move, third_person_camera).chain())
        .add_systems(Update, update_performance_overlay)
        .run();
}

#[derive(Component)]
struct Player {
    walk_speed: f32,
    sprint_speed: f32,
    turn_speed: f32,
    jump_speed: f32,
    gravity: f32,
}

#[derive(Component)]
struct ThirdPersonCameraRig {
    yaw: f32,
    pitch: f32,
    look_sensitivity: f32,
    zoom_sensitivity: f32,
    distance: f32,
    min_distance: f32,
    max_distance: f32,
    height: f32,
    focus_height: f32,
}

#[derive(Component)]
struct PerformanceOverlayText;

#[derive(Component, Clone, Copy)]
struct PlayerCollider {
    half_extents: Vec3,
}

#[derive(Component, Clone, Copy)]
struct WorldCollider {
    half_extents: Vec3,
}

#[derive(Component, Default)]
struct PlayerKinematics {
    vertical_velocity: f32,
    grounded: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            walk_speed: 5.5,
            sprint_speed: 9.5,
            turn_speed: 2.8,
            jump_speed: 7.5,
            gravity: -20.0,
        }
    }
}

impl Default for ThirdPersonCameraRig {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: -0.35,
            look_sensitivity: 0.0025,
            zoom_sensitivity: 0.35,
            distance: 8.0,
            min_distance: 2.5,
            max_distance: 20.0,
            height: 2.0,
            focus_height: 1.1,
        }
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_mesh = meshes.add(Cuboid::new(0.8, 1.8, 0.8));
    let player_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.84, 0.82, 0.7),
        perceptual_roughness: 0.8,
        ..default()
    });

    commands.spawn((
        Player::default(),
        Mesh3d(player_mesh),
        MeshMaterial3d(player_mat),
        Transform::from_xyz(0.0, 0.9, 0.0),
        PlayerCollider {
            half_extents: Vec3::new(0.35, 0.9, 0.35),
        },
        PlayerKinematics {
            vertical_velocity: 0.0,
            grounded: true,
        },
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 4.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ThirdPersonCameraRig::default(),
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 25_000.0,
            ..default()
        },
        Transform::from_xyz(18.0, 24.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let ground_mesh = meshes.add(Cuboid::new(120.0, 0.1, 120.0));
    let ground_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.17, 0.35, 0.18),
        perceptual_roughness: 1.0,
        ..default()
    });

    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(ground_mat),
        Transform::from_xyz(0.0, -0.05, 0.0),
        WorldCollider {
            half_extents: Vec3::new(60.0, 0.05, 60.0),
        },
    ));

    let wall_mesh = meshes.add(Cuboid::new(3.0, 3.0, 3.0));
    let tower_mesh = meshes.add(Cuboid::new(4.0, 8.0, 4.0));
    let crate_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.48, 0.44, 0.4),
        perceptual_roughness: 0.95,
        ..default()
    });
    let tower_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.31, 0.42),
        metallic: 0.05,
        perceptual_roughness: 0.75,
        ..default()
    });
    let crate_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.58, 0.36, 0.2),
        perceptual_roughness: 0.9,
        ..default()
    });

    for x in -8..=8 {
        for z in -8..=8 {
            let near_spawn = (-1..=1).contains(&x) && (-1..=1).contains(&z);
            if (x + z) % 4 == 0 && !near_spawn {
                commands.spawn((
                    Mesh3d(crate_mesh.clone()),
                    MeshMaterial3d(crate_mat.clone()),
                    Transform::from_xyz(x as f32 * 3.0, 0.5, z as f32 * 3.0),
                    WorldCollider {
                        half_extents: Vec3::splat(0.5),
                    },
                ));
            }
        }
    }

    for i in -5..=5 {
        commands.spawn((
            Mesh3d(wall_mesh.clone()),
            MeshMaterial3d(wall_mat.clone()),
            Transform::from_xyz(i as f32 * 3.2, 1.5, -20.0),
            WorldCollider {
                half_extents: Vec3::splat(1.5),
            },
        ));
    }

    commands.spawn((
        Mesh3d(tower_mesh),
        MeshMaterial3d(tower_mat),
        Transform::from_xyz(0.0, 4.0, -30.0),
        WorldCollider {
            half_extents: Vec3::new(2.0, 4.0, 2.0),
        },
    ));

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        })
        .with_child(Text::new(
            "Tank controls:\nW/S vooruit-achteruit\nQ/E zijwaarts\nA/D draaien\n\
             Rechtermuisknop ingedrukt:\nA/D/Q/E zijwaarts en character kijkt met camera mee\n\
             Shift: Sprint\nSpace: Springen\nLMB: camera orbit\nScroll: zoom in/uit",
        ));

    commands.spawn((
        PerformanceOverlayText,
        Text::new("FPS: --\nFrame time: -- ms"),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            right: px(12),
            ..default()
        },
    ));
}

fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    camera_query: Query<&ThirdPersonCameraRig, With<Camera3d>>,
    mut player_query: Query<(&mut Transform, &Player, &PlayerCollider, &mut PlayerKinematics)>,
    world_colliders: Query<(&Transform, &WorldCollider), Without<Player>>,
) {
    let Ok(camera_rig) = camera_query.single() else {
        return;
    };

    let Ok((mut transform, player, player_collider, mut kinematics)) = player_query.single_mut()
    else {
        return;
    };

    let rmb_held = mouse_buttons.pressed(MouseButton::Right);
    if rmb_held {
        transform.rotation = Quat::from_rotation_y(camera_rig.yaw);
    }

    let dt = time.delta_secs();
    if !rmb_held {
        let turn_axis = (keys.pressed(KeyCode::KeyD) as i8 - keys.pressed(KeyCode::KeyA) as i8) as f32;
        if turn_axis != 0.0 {
            transform.rotate_y(-turn_axis * player.turn_speed * dt);
        }
    }

    let forward = transform.rotation * -Vec3::Z;
    let right = transform.rotation * Vec3::X;

    let forward_axis = (keys.pressed(KeyCode::KeyW) as i8 - keys.pressed(KeyCode::KeyS) as i8) as f32;
    let strafe_axis = if rmb_held {
        let strafe_right = keys.pressed(KeyCode::KeyE) || keys.pressed(KeyCode::KeyD);
        let strafe_left = keys.pressed(KeyCode::KeyQ) || keys.pressed(KeyCode::KeyA);
        (strafe_right as i8 - strafe_left as i8) as f32
    } else {
        (keys.pressed(KeyCode::KeyE) as i8 - keys.pressed(KeyCode::KeyQ) as i8) as f32
    };

    let movement = (forward * forward_axis + right * strafe_axis).normalize_or_zero();

    let speed = if keys.pressed(KeyCode::ShiftLeft) {
        player.sprint_speed
    } else {
        player.walk_speed
    };

    let desired_delta = movement * speed * dt;

    let mut next_position = transform.translation;

    if desired_delta.x != 0.0 {
        let candidate = Vec3::new(next_position.x + desired_delta.x, next_position.y, next_position.z);
        if !would_collide(candidate, player_collider.half_extents, &world_colliders) {
            next_position.x = candidate.x;
        }
    }

    if desired_delta.z != 0.0 {
        let candidate = Vec3::new(next_position.x, next_position.y, next_position.z + desired_delta.z);
        if !would_collide(candidate, player_collider.half_extents, &world_colliders) {
            next_position.z = candidate.z;
        }
    }

    if keys.just_pressed(KeyCode::Space) && kinematics.grounded {
        kinematics.vertical_velocity = player.jump_speed;
        kinematics.grounded = false;
    }

    let vertical_start = next_position;
    kinematics.vertical_velocity += player.gravity * dt;
    let proposed_vertical = Vec3::new(
        vertical_start.x,
        vertical_start.y + kinematics.vertical_velocity * dt,
        vertical_start.z,
    );

    if kinematics.vertical_velocity <= 0.0 {
        if let Some(landing_top) = find_landing_top(
            vertical_start,
            proposed_vertical,
            player_collider.half_extents,
            &world_colliders,
        ) {
            next_position.y = landing_top + player_collider.half_extents.y;
            kinematics.vertical_velocity = 0.0;
            kinematics.grounded = true;
        } else {
            next_position.y = proposed_vertical.y;
            kinematics.grounded = false;
        }
    } else if let Some(ceiling_bottom) = find_ceiling_bottom(
        vertical_start,
        proposed_vertical,
        player_collider.half_extents,
        &world_colliders,
    ) {
        next_position.y = ceiling_bottom - player_collider.half_extents.y;
        kinematics.vertical_velocity = 0.0;
        kinematics.grounded = false;
    } else {
        next_position.y = proposed_vertical.y;
        kinematics.grounded = false;
    }

    transform.translation = next_position;
}

fn third_person_camera(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    player_query: Query<&Transform, (With<Player>, Without<Camera3d>)>,
    mut camera_query: Query<(&mut Transform, &mut ThirdPersonCameraRig), With<Camera3d>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let Ok((mut camera_transform, mut rig)) = camera_query.single_mut() else {
        return;
    };

    let orbit_pressed =
        mouse_buttons.pressed(MouseButton::Left) || mouse_buttons.pressed(MouseButton::Right);
    if orbit_pressed {
        let mouse_delta = mouse_motion.delta;
        rig.yaw -= mouse_delta.x * rig.look_sensitivity;
        rig.pitch -= mouse_delta.y * rig.look_sensitivity;
        rig.pitch = rig.pitch.clamp(-1.2, 0.6);
    }
    rig.distance = (rig.distance - mouse_scroll.delta.y * rig.zoom_sensitivity)
        .clamp(rig.min_distance, rig.max_distance);

    let target = player_transform.translation;
    let rotation = Quat::from_euler(EulerRot::YXZ, rig.yaw, rig.pitch, 0.0);
    let orbit_offset = rotation * Vec3::new(0.0, 0.0, rig.distance);

    camera_transform.translation = target + orbit_offset + Vec3::Y * rig.height;
    camera_transform.look_at(target + Vec3::Y * rig.focus_height, Vec3::Y);
}

fn update_performance_overlay(
    diagnostics: Res<DiagnosticsStore>,
    mut text_query: Query<&mut Text, With<PerformanceOverlayText>>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or(0.0);

    let frame_time_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|frame_time| frame_time.smoothed())
        .unwrap_or(0.0);

    for mut text in &mut text_query {
        **text = format!("FPS: {fps:>6.1}\nFrame time: {frame_time_ms:>6.2} ms");
    }
}

fn would_collide(
    player_center: Vec3,
    player_half_extents: Vec3,
    world_colliders: &Query<(&Transform, &WorldCollider), Without<Player>>,
) -> bool {
    world_colliders.iter().any(|(world_transform, world_collider)| {
        intersects_aabb(
            player_center,
            player_half_extents,
            world_transform.translation,
            world_collider.half_extents,
        )
    })
}

fn find_landing_top(
    previous_center: Vec3,
    proposed_center: Vec3,
    player_half_extents: Vec3,
    world_colliders: &Query<(&Transform, &WorldCollider), Without<Player>>,
) -> Option<f32> {
    let previous_bottom = previous_center.y - player_half_extents.y;
    let proposed_bottom = proposed_center.y - player_half_extents.y;
    let epsilon = 0.0001;

    world_colliders
        .iter()
        .filter_map(|(world_transform, world_collider)| {
            if !intersects_xz(
                proposed_center,
                player_half_extents,
                world_transform.translation,
                world_collider.half_extents,
            ) {
                return None;
            }

            let collider_top = world_transform.translation.y + world_collider.half_extents.y;
            let crossed_top =
                previous_bottom >= collider_top - epsilon && proposed_bottom <= collider_top + epsilon;

            crossed_top.then_some(collider_top)
        })
        .max_by(f32::total_cmp)
}

fn find_ceiling_bottom(
    previous_center: Vec3,
    proposed_center: Vec3,
    player_half_extents: Vec3,
    world_colliders: &Query<(&Transform, &WorldCollider), Without<Player>>,
) -> Option<f32> {
    let previous_top = previous_center.y + player_half_extents.y;
    let proposed_top = proposed_center.y + player_half_extents.y;
    let epsilon = 0.0001;

    world_colliders
        .iter()
        .filter_map(|(world_transform, world_collider)| {
            if !intersects_xz(
                proposed_center,
                player_half_extents,
                world_transform.translation,
                world_collider.half_extents,
            ) {
                return None;
            }

            let collider_bottom = world_transform.translation.y - world_collider.half_extents.y;
            let crossed_bottom =
                previous_top <= collider_bottom + epsilon && proposed_top >= collider_bottom - epsilon;

            crossed_bottom.then_some(collider_bottom)
        })
        .min_by(f32::total_cmp)
}

fn intersects_xz(a_center: Vec3, a_half_extents: Vec3, b_center: Vec3, b_half_extents: Vec3) -> bool {
    (a_center.x - b_center.x).abs() < (a_half_extents.x + b_half_extents.x)
        && (a_center.z - b_center.z).abs() < (a_half_extents.z + b_half_extents.z)
}

fn intersects_aabb(a_center: Vec3, a_half_extents: Vec3, b_center: Vec3, b_half_extents: Vec3) -> bool {
    (a_center.x - b_center.x).abs() < (a_half_extents.x + b_half_extents.x)
        && (a_center.y - b_center.y).abs() < (a_half_extents.y + b_half_extents.y)
        && (a_center.z - b_center.z).abs() < (a_half_extents.z + b_half_extents.z)
}
