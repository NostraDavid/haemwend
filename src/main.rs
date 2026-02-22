use bevy::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::window::{PresentMode, WindowResolution};

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
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.6, 0.65, 0.7),
            brightness: 250.0,
            affects_lightmapped_meshes: true,
        })
        .add_systems(Startup, setup_world)
        .add_systems(Update, (player_move, third_person_camera).chain())
        .run();
}

#[derive(Component)]
struct Player {
    walk_speed: f32,
    sprint_speed: f32,
}

#[derive(Component)]
struct ThirdPersonCameraRig {
    yaw: f32,
    pitch: f32,
    look_sensitivity: f32,
    distance: f32,
    height: f32,
    focus_height: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            walk_speed: 5.5,
            sprint_speed: 9.5,
        }
    }
}

impl Default for ThirdPersonCameraRig {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: -0.35,
            look_sensitivity: 0.0025,
            distance: 8.0,
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
            if (x + z) % 4 == 0 {
                commands.spawn((
                    Mesh3d(crate_mesh.clone()),
                    MeshMaterial3d(crate_mat.clone()),
                    Transform::from_xyz(x as f32 * 3.0, 0.5, z as f32 * 3.0),
                ));
            }
        }
    }

    for i in -5..=5 {
        commands.spawn((
            Mesh3d(wall_mesh.clone()),
            MeshMaterial3d(wall_mat.clone()),
            Transform::from_xyz(i as f32 * 3.2, 1.5, -20.0),
        ));
    }

    commands.spawn((
        Mesh3d(tower_mesh),
        MeshMaterial3d(tower_mat),
        Transform::from_xyz(0.0, 4.0, -30.0),
    ));

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        })
        .with_child(Text::new(
            "W/A/S/D: Move\nShift: Sprint\nHold Right Mouse: Orbit camera",
        ));
}

fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    camera_query: Query<&ThirdPersonCameraRig, With<Camera3d>>,
    mut player_query: Query<(&mut Transform, &Player)>,
) {
    let Ok(camera_rig) = camera_query.single() else {
        return;
    };

    let Ok((mut transform, player)) = player_query.single_mut() else {
        return;
    };

    let yaw_rotation = Quat::from_rotation_y(camera_rig.yaw);
    let forward = yaw_rotation * -Vec3::Z;
    let right = yaw_rotation * Vec3::X;

    let mut movement = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        movement += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        movement -= forward;
    }
    if keys.pressed(KeyCode::KeyA) {
        movement -= right;
    }
    if keys.pressed(KeyCode::KeyD) {
        movement += right;
    }

    let speed = if keys.pressed(KeyCode::ShiftLeft) {
        player.sprint_speed
    } else {
        player.walk_speed
    };

    let move_dir = movement.normalize_or_zero();
    transform.translation += move_dir * speed * time.delta_secs();

    if move_dir != Vec3::ZERO {
        let yaw = move_dir.x.atan2(-move_dir.z);
        transform.rotation = Quat::from_rotation_y(yaw);
    }
}

fn third_person_camera(
    mouse_motion: Res<AccumulatedMouseMotion>,
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

    if mouse_buttons.pressed(MouseButton::Right) {
        let mouse_delta = mouse_motion.delta;
        rig.yaw -= mouse_delta.x * rig.look_sensitivity;
        rig.pitch -= mouse_delta.y * rig.look_sensitivity;
        rig.pitch = rig.pitch.clamp(-1.2, 0.6);
    }

    let target = player_transform.translation;
    let rotation = Quat::from_euler(EulerRot::YXZ, rig.yaw, rig.pitch, 0.0);
    let orbit_offset = rotation * Vec3::new(0.0, 0.0, rig.distance);

    camera_transform.translation = target + orbit_offset + Vec3::Y * rig.height;
    camera_transform.look_at(target + Vec3::Y * rig.focus_height, Vec3::Y);
}
