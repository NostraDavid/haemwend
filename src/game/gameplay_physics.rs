use super::*;

pub(super) fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    menu: Res<MenuState>,
    keybinds: Res<GameKeybinds>,
    camera_query: Query<&ThirdPersonCameraRig, With<Camera3d>>,
    mut player_query: Query<(
        &mut Transform,
        &Player,
        &PlayerCollider,
        &mut PlayerKinematics,
    )>,
    world_colliders: Query<(&Transform, &WorldCollider), Without<Player>>,
) {
    if menu.open {
        return;
    }

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
        let turn_axis = (keybinds.action_pressed(&keys, GameAction::TurnRight) as i8
            - keybinds.action_pressed(&keys, GameAction::TurnLeft) as i8)
            as f32;
        if turn_axis != 0.0 {
            transform.rotate_y(-turn_axis * player.turn_speed * dt);
        }
    }

    let forward = transform.rotation * -Vec3::Z;
    let right = transform.rotation * Vec3::X;

    let forward_axis = (keybinds.action_pressed(&keys, GameAction::MoveForward) as i8
        - keybinds.action_pressed(&keys, GameAction::MoveBackward) as i8)
        as f32;

    let strafe_axis = if rmb_held {
        let strafe_right = keybinds.action_pressed(&keys, GameAction::StrafeRight)
            || keybinds.action_pressed(&keys, GameAction::TurnRight);
        let strafe_left = keybinds.action_pressed(&keys, GameAction::StrafeLeft)
            || keybinds.action_pressed(&keys, GameAction::TurnLeft);
        (strafe_right as i8 - strafe_left as i8) as f32
    } else {
        (keybinds.action_pressed(&keys, GameAction::StrafeRight) as i8
            - keybinds.action_pressed(&keys, GameAction::StrafeLeft) as i8) as f32
    };

    let movement = (forward * forward_axis + right * strafe_axis).normalize_or_zero();

    let speed = if keybinds.action_pressed(&keys, GameAction::Sprint) {
        player.sprint_speed
    } else {
        player.walk_speed
    };

    let desired_delta = movement * speed * dt;

    let mut next_position = transform.translation;

    if desired_delta.x != 0.0 {
        let candidate = Vec3::new(
            next_position.x + desired_delta.x,
            next_position.y,
            next_position.z,
        );
        if !would_collide(candidate, player_collider.half_extents, &world_colliders) {
            next_position.x = candidate.x;
        }
    }

    if desired_delta.z != 0.0 {
        let candidate = Vec3::new(
            next_position.x,
            next_position.y,
            next_position.z + desired_delta.z,
        );
        if !would_collide(candidate, player_collider.half_extents, &world_colliders) {
            next_position.z = candidate.z;
        }
    }

    if keybinds.action_just_pressed(&keys, GameAction::Jump) && kinematics.grounded {
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

pub(super) fn third_person_camera(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    menu: Res<MenuState>,
    player_query: Query<&Transform, (With<Player>, Without<Camera3d>)>,
    mut camera_query: Query<(&mut Transform, &mut ThirdPersonCameraRig), With<Camera3d>>,
) {
    if menu.open {
        return;
    }

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

pub(super) fn update_player_blob_shadow(
    settings: Res<GameSettings>,
    player_query: Query<(&Transform, &PlayerCollider), (With<Player>, Without<PlayerBlobShadow>)>,
    world_colliders: Query<(&GlobalTransform, &WorldCollider), Without<Player>>,
    mut shadow_query: Query<
        (&mut Transform, &MeshMaterial3d<StandardMaterial>),
        (With<PlayerBlobShadow>, Without<Player>),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if settings.shadow_mode != ShadowModeSetting::Blob {
        return;
    }

    let Ok((player_transform, player_collider)) = player_query.single() else {
        return;
    };
    let Ok((mut shadow_transform, shadow_material)) = shadow_query.single_mut() else {
        return;
    };

    let player_pos = player_transform.translation;
    let support_top = world_colliders
        .iter()
        .filter_map(|(world_transform, world_collider)| {
            let world_pos = world_transform.translation();
            let inside_x =
                (player_pos.x - world_pos.x).abs() <= world_collider.half_extents.x + 0.45;
            let inside_z =
                (player_pos.z - world_pos.z).abs() <= world_collider.half_extents.z + 0.45;
            if !inside_x || !inside_z {
                return None;
            }

            let top = world_pos.y + world_collider.half_extents.y;
            (top <= player_pos.y + 0.2).then_some(top)
        })
        .max_by(f32::total_cmp)
        .unwrap_or(0.0);

    let feet_height = player_pos.y - player_collider.half_extents.y;
    let hover_height = (feet_height - support_top).max(0.0);
    let fade = (1.0 - hover_height / 6.0).clamp(0.0, 1.0);
    let radius = (0.95 - hover_height * 0.08).clamp(0.55, 0.95);

    shadow_transform.translation = Vec3::new(player_pos.x, support_top + 0.015, player_pos.z);
    shadow_transform.scale = Vec3::new(radius, 1.0, radius);

    if let Some(material) = materials.get_mut(&shadow_material.0) {
        material.base_color = material.base_color.with_alpha(0.16 + 0.42 * fade);
    }
}

pub(super) fn update_performance_overlay(
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

pub(super) fn sync_mouse_capture_with_focus(
    flow: Res<GameFlowState>,
    menu: Res<MenuState>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_capture_state: ResMut<MouseLookCaptureState>,
    window_query: Single<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    let (mut window, mut cursor_options) = window_query.into_inner();
    let look_held =
        mouse_buttons.pressed(MouseButton::Left) || mouse_buttons.pressed(MouseButton::Right);

    if window.focused && flow.in_game && !menu.open {
        if look_held {
            if !mouse_capture_state.active {
                mouse_capture_state.restore_position = window.cursor_position();
                mouse_capture_state.active = true;
            }
            cursor_options.visible = false;
            cursor_options.grab_mode = CursorGrabMode::Locked;
        } else {
            if mouse_capture_state.active {
                if let Some(position) = mouse_capture_state.restore_position.take() {
                    window.set_cursor_position(Some(position));
                }
                mouse_capture_state.active = false;
            }
            cursor_options.visible = true;
            cursor_options.grab_mode = CursorGrabMode::Confined;
        }
    } else {
        mouse_capture_state.active = false;
        mouse_capture_state.restore_position = None;
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
}

pub(super) fn menu_button_node() -> Node {
    Node {
        width: percent(100),
        height: px(40),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        margin: UiRect::bottom(px(6)),
        ..default()
    }
}

pub(super) fn menu_button_normal_color() -> BackgroundColor {
    BackgroundColor(Color::srgb(0.17, 0.20, 0.26))
}

pub(super) fn menu_button_hover_color() -> BackgroundColor {
    BackgroundColor(Color::srgb(0.23, 0.27, 0.35))
}

pub(super) fn menu_button_pressed_color() -> BackgroundColor {
    BackgroundColor(Color::srgb(0.19, 0.43, 0.25))
}

pub(super) fn spawn_baked_shadow(
    commands: &mut Commands,
    shadow_mesh: &Handle<Mesh>,
    shadow_material: &Handle<StandardMaterial>,
    center: Vec3,
    size: Vec2,
) {
    commands.spawn((
        BakedShadow,
        InGameEntity,
        Mesh3d(shadow_mesh.clone()),
        MeshMaterial3d(shadow_material.clone()),
        Transform::from_translation(center).with_scale(Vec3::new(size.x, 1.0, size.y)),
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

pub(super) fn would_collide(
    player_center: Vec3,
    player_half_extents: Vec3,
    world_colliders: &Query<(&Transform, &WorldCollider), Without<Player>>,
) -> bool {
    world_colliders
        .iter()
        .any(|(world_transform, world_collider)| {
            intersects_aabb(
                player_center,
                player_half_extents,
                world_transform.translation,
                world_collider.half_extents,
            )
        })
}

pub(super) fn find_landing_top(
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
            let crossed_top = previous_bottom >= collider_top - epsilon
                && proposed_bottom <= collider_top + epsilon;

            crossed_top.then_some(collider_top)
        })
        .max_by(f32::total_cmp)
}

pub(super) fn find_ceiling_bottom(
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
            let crossed_bottom = previous_top <= collider_bottom + epsilon
                && proposed_top >= collider_bottom - epsilon;

            crossed_bottom.then_some(collider_bottom)
        })
        .min_by(f32::total_cmp)
}

pub(super) fn intersects_xz(
    a_center: Vec3,
    a_half_extents: Vec3,
    b_center: Vec3,
    b_half_extents: Vec3,
) -> bool {
    (a_center.x - b_center.x).abs() < (a_half_extents.x + b_half_extents.x)
        && (a_center.z - b_center.z).abs() < (a_half_extents.z + b_half_extents.z)
}

pub(super) fn intersects_aabb(
    a_center: Vec3,
    a_half_extents: Vec3,
    b_center: Vec3,
    b_half_extents: Vec3,
) -> bool {
    (a_center.x - b_center.x).abs() < (a_half_extents.x + b_half_extents.x)
        && (a_center.y - b_center.y).abs() < (a_half_extents.y + b_half_extents.y)
        && (a_center.z - b_center.z).abs() < (a_half_extents.z + b_half_extents.z)
}
