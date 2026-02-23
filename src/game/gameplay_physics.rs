use super::*;

const CONTROLLER_MAX_SLIDES: usize = 4;
const CONTROLLER_SKIN: f32 = 0.02;
const CONTROLLER_STEP_HEIGHT: f32 = 0.38;
const CONTROLLER_STEP_DROP: f32 = 0.25;

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
    world_collision_grid: Res<WorldCollisionGrid>,
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
    let (slid_position, blocked) = move_with_slide(
        next_position,
        desired_delta,
        *player_collider,
        &world_collision_grid,
        CONTROLLER_MAX_SLIDES,
        CONTROLLER_SKIN,
    );
    next_position.x = slid_position.x;
    next_position.z = slid_position.z;

    if blocked && kinematics.grounded {
        if let Some(step_position) = try_step_move(
            transform.translation,
            desired_delta,
            *player_collider,
            &world_collision_grid,
            CONTROLLER_STEP_HEIGHT,
            CONTROLLER_STEP_DROP,
            CONTROLLER_SKIN,
        ) {
            next_position = step_position;
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
            *player_collider,
            &world_collision_grid,
        ) {
            next_position.y = landing_top + player_collider.half_height;
            kinematics.vertical_velocity = 0.0;
            kinematics.grounded = true;
        } else {
            next_position.y = proposed_vertical.y;
            kinematics.grounded = false;
        }
    } else if let Some(ceiling_bottom) = find_ceiling_bottom(
        vertical_start,
        proposed_vertical,
        *player_collider,
        &world_collision_grid,
    ) {
        next_position.y = ceiling_bottom - player_collider.half_height;
        kinematics.vertical_velocity = 0.0;
        kinematics.grounded = false;
    } else {
        next_position.y = proposed_vertical.y;
        kinematics.grounded = false;
    }

    transform.translation = next_position;
}

pub(super) fn animate_procedural_human(
    time: Res<Time>,
    menu: Res<MenuState>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    world_collision_grid: Res<WorldCollisionGrid>,
    camera_query: Query<&ThirdPersonCameraRig, With<Camera3d>>,
    mut player_query: Query<
        (&Transform, &mut ProceduralHumanAnimState),
        (
            With<Player>,
            Without<ProceduralHumanVisualRoot>,
            Without<HumanLegHip>,
            Without<HumanLegKnee>,
            Without<HumanArmPivot>,
            Without<HumanHead>,
        ),
    >,
    mut visual_root_query: Query<
        &mut Transform,
        (
            With<ProceduralHumanVisualRoot>,
            Without<Player>,
            Without<HumanLegHip>,
            Without<HumanLegKnee>,
            Without<HumanArmPivot>,
            Without<HumanHead>,
        ),
    >,
    mut leg_hips: Query<
        (&HumanLegHip, &mut Transform, &Children),
        (
            Without<Player>,
            Without<ProceduralHumanVisualRoot>,
            Without<HumanLegKnee>,
            Without<HumanArmPivot>,
            Without<HumanHead>,
        ),
    >,
    mut leg_knees: Query<
        &mut Transform,
        (
            With<HumanLegKnee>,
            Without<Player>,
            Without<ProceduralHumanVisualRoot>,
            Without<HumanLegHip>,
            Without<HumanArmPivot>,
            Without<HumanHead>,
        ),
    >,
    mut arm_pivots: Query<
        (&HumanArmPivot, &mut Transform),
        (
            Without<Player>,
            Without<ProceduralHumanVisualRoot>,
            Without<HumanLegHip>,
            Without<HumanLegKnee>,
            Without<HumanHead>,
        ),
    >,
    mut heads: Query<
        (&HumanHead, &mut Transform),
        (
            With<HumanHead>,
            Without<Player>,
            Without<ProceduralHumanVisualRoot>,
            Without<HumanLegHip>,
            Without<HumanLegKnee>,
            Without<HumanArmPivot>,
        ),
    >,
) {
    let Ok((player_transform, mut anim_state)) = player_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs().max(1e-5);
    let delta = player_transform.translation - anim_state.last_position;
    let measured_speed = Vec2::new(delta.x, delta.z).length() / dt;
    anim_state.last_position = player_transform.translation;
    let target_visual_y = player_transform.translation.y;
    let vertical_follow_rate = if target_visual_y > anim_state.visual_center_y {
        9.0
    } else {
        16.0
    };
    let vertical_follow = 1.0 - (-dt * vertical_follow_rate).exp();
    anim_state.visual_center_y += (target_visual_y - anim_state.visual_center_y) * vertical_follow;

    let target_speed = if menu.open { 0.0 } else { measured_speed };
    let smooth = 1.0 - (-dt * 10.0).exp();
    anim_state.smoothed_speed += (target_speed - anim_state.smoothed_speed) * smooth;
    let speed_factor = (anim_state.smoothed_speed / 8.0).clamp(0.0, 1.0);

    anim_state.phase += dt * (2.0 + anim_state.smoothed_speed * 2.0);
    if anim_state.phase > std::f32::consts::TAU {
        anim_state.phase -= std::f32::consts::TAU;
    }

    let stride_bob = (anim_state.phase * 2.0).sin() * (0.01 + 0.045 * speed_factor);
    let idle_bob = (time.elapsed_secs() * 1.5).sin() * (0.006 * (1.0 - speed_factor));
    let lean_roll = (anim_state.phase).sin() * 0.06 * speed_factor;
    let mut root_local_translation = Vec3::new(0.0, -0.9 + stride_bob + idle_bob, 0.0);
    let root_local_rotation =
        Quat::from_rotation_y(std::f32::consts::PI) * Quat::from_rotation_z(lean_roll);
    let root_world_rotation = player_transform.rotation * root_local_rotation;
    let visual_player_translation = Vec3::new(
        player_transform.translation.x,
        anim_state.visual_center_y,
        player_transform.translation.z,
    );
    let mut root_world_translation =
        visual_player_translation + player_transform.rotation * root_local_translation;

    let mut head_yaw_target = 0.0;
    let mut head_pitch_target = 0.0;
    if mouse_buttons.pressed(MouseButton::Left) {
        if let Ok(camera_rig) = camera_query.single() {
            let (player_yaw, _, _) = player_transform.rotation.to_euler(EulerRot::YXZ);
            head_yaw_target = shortest_angle_delta(player_yaw, camera_rig.yaw);
            head_pitch_target = -camera_rig.pitch;
        }
    }

    let gait = smoothstep01(((speed_factor - 0.10) / 0.25).clamp(0.0, 1.0));

    // If one foot is supported lower (edge of stairs), lower pelvis so stance feet can reach.
    let mut pelvis_drop = 0.0_f32;
    for _ in 0..2 {
        let test_root = root_world_translation - Vec3::Y * pelvis_drop;
        let mut required_drop = 0.0_f32;

        for (hip, _, _) in &mut leg_hips {
            let (swing, lift, stride) = leg_motion(anim_state.phase, hip.side, gait);
            let nominal_local = hip.base_local
                + Vec3::new(
                    0.0,
                    -(hip.upper_len + hip.lower_len) + lift * (0.10 + 0.08 * gait),
                    stride,
                );
            let mut ankle_target_world = test_root + root_world_rotation * nominal_local;
            let probe = Vec3::new(
                ankle_target_world.x,
                test_root.y + 2.0,
                ankle_target_world.z,
            );

            if let Some(ground_y) = sample_ground_height(&world_collision_grid, probe, 0.12) {
                let planted_y = ground_y + hip.ankle_height;
                let stance = 1.0 - lift;
                let plant_strength = (0.82 + (1.0 - gait) * 0.16).clamp(0.0, 0.98);
                ankle_target_world.y = ankle_target_world.y.max(planted_y);
                ankle_target_world.y = ankle_target_world.y * (1.0 - stance * plant_strength)
                    + planted_y * (stance * plant_strength);

                let target_local = root_world_rotation.inverse() * (ankle_target_world - test_root);
                let to_target = target_local - hip.base_local;
                let dy = to_target.y;
                let dz = to_target.z;
                let leg_total = hip.upper_len + hip.lower_len;
                let max_reach = (leg_total - 0.015).max(0.05);
                if dz.abs() >= max_reach {
                    continue;
                }

                let reachable_dy = -(max_reach * max_reach - dz * dz).sqrt();
                let needed = (reachable_dy - dy).max(0.0);
                required_drop = required_drop.max(needed * (1.0 - 0.35 * swing.abs()));
            }
        }

        if required_drop <= 0.0005 {
            break;
        }
        pelvis_drop = (pelvis_drop + required_drop).min(0.35);
    }

    if pelvis_drop > 0.0 {
        root_local_translation.y -= pelvis_drop;
        root_world_translation =
            visual_player_translation + player_transform.rotation * root_local_translation;
    }

    if let Ok(mut root_transform) = visual_root_query.single_mut() {
        root_transform.translation = root_local_translation;
        root_transform.rotation = root_local_rotation;
    }

    for (hip, mut hip_transform, children) in &mut leg_hips {
        let (_swing, lift, stride) = leg_motion(anim_state.phase, hip.side, gait);

        let nominal_local = hip.base_local
            + Vec3::new(
                0.0,
                -(hip.upper_len + hip.lower_len) + lift * (0.10 + 0.08 * gait),
                stride,
            );
        let mut ankle_target_world = root_world_translation + root_world_rotation * nominal_local;

        let probe = Vec3::new(
            ankle_target_world.x,
            root_world_translation.y + 2.0,
            ankle_target_world.z,
        );
        if let Some(ground_y) = sample_ground_height(&world_collision_grid, probe, 0.12) {
            let planted_y = ground_y + hip.ankle_height;
            let stance = 1.0 - lift;
            let plant_strength = (0.82 + (1.0 - gait) * 0.16).clamp(0.0, 0.98);
            ankle_target_world.y = ankle_target_world.y.max(planted_y);
            ankle_target_world.y = ankle_target_world.y * (1.0 - stance * plant_strength)
                + planted_y * (stance * plant_strength);
        }

        let target_local =
            root_world_rotation.inverse() * (ankle_target_world - root_world_translation);
        let to_target = target_local - hip.base_local;
        let dy = to_target.y;
        let dz = to_target.z;

        let leg_total = hip.upper_len + hip.lower_len;
        let dist = (dy * dy + dz * dz).sqrt().clamp(0.05, leg_total - 0.001);
        let base_angle = dz.atan2(-dy);
        let cos_hip = ((hip.upper_len * hip.upper_len + dist * dist
            - hip.lower_len * hip.lower_len)
            / (2.0 * hip.upper_len * dist))
            .clamp(-1.0, 1.0);
        let hip_pitch = base_angle - cos_hip.acos();
        let cos_knee = ((hip.upper_len * hip.upper_len + hip.lower_len * hip.lower_len
            - dist * dist)
            / (2.0 * hip.upper_len * hip.lower_len))
            .clamp(-1.0, 1.0);
        let knee_pitch = std::f32::consts::PI - cos_knee.acos();

        hip_transform.translation = hip.base_local;
        hip_transform.rotation = Quat::from_euler(EulerRot::XYZ, hip_pitch, 0.0, 0.0);

        for child in children {
            if let Ok(mut knee_transform) = leg_knees.get_mut(*child) {
                knee_transform.translation = Vec3::new(0.0, -hip.upper_len, 0.0);
                knee_transform.rotation = Quat::from_euler(EulerRot::XYZ, knee_pitch, 0.0, 0.0);
            }
        }
    }

    for (pivot, mut transform) in &mut arm_pivots {
        let side_phase = if pivot.side == LimbSide::Left {
            std::f32::consts::PI
        } else {
            0.0
        };
        let swing = (anim_state.phase + side_phase).sin();
        let idle = (time.elapsed_secs() * 1.8 + side_phase).sin() * 0.07 * (1.0 - speed_factor);
        let pitch = swing * (0.15 + 0.72 * speed_factor) + idle;
        transform.translation = pivot.base_local;
        transform.rotation = Quat::from_euler(EulerRot::XYZ, pitch, 0.0, 0.0);
    }

    let head_blend = 1.0 - (-dt * 12.0).exp();
    for (head, mut transform) in &mut heads {
        let yaw = head_yaw_target.clamp(-head.max_yaw, head.max_yaw);
        let pitch = head_pitch_target.clamp(-head.max_pitch_down, head.max_pitch_up);
        let target_rot = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
        transform.translation = head.base_local;
        transform.rotation = transform.rotation.slerp(target_rot, head_blend);
    }
}

fn shortest_angle_delta(from: f32, to: f32) -> f32 {
    let mut delta =
        (to - from + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
    if delta <= -std::f32::consts::PI {
        delta += std::f32::consts::TAU;
    }
    delta
}

fn smoothstep01(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn leg_motion(phase: f32, side: LimbSide, gait: f32) -> (f32, f32, f32) {
    let side_phase = if side == LimbSide::Left {
        0.0
    } else {
        std::f32::consts::PI
    };
    let swing = (phase + side_phase).sin();
    let lift = swing.max(0.0) * gait;
    let stride = swing * (0.22 * gait);
    (swing, lift, stride)
}

fn sample_ground_height(
    grid: &WorldCollisionGrid,
    probe_world: Vec3,
    foot_radius: f32,
) -> Option<f32> {
    let mut best_top: Option<f32> = None;
    grid.query_nearby(probe_world, foot_radius + 0.2, |collider| {
        if !intersects_disc_aabb_xz(
            probe_world,
            foot_radius,
            collider.center,
            collider.half_extents,
        ) {
            return;
        }

        let top = collider.center.y + collider.half_extents.y;
        if top <= probe_world.y {
            best_top = Some(best_top.map_or(top, |current| current.max(top)));
        }
    });
    best_top
}

fn move_with_slide(
    start: Vec3,
    displacement: Vec3,
    collider: PlayerCollider,
    grid: &WorldCollisionGrid,
    max_iterations: usize,
    skin: f32,
) -> (Vec3, bool) {
    let mut position = start;
    let mut remaining = Vec2::new(displacement.x, displacement.z);
    let mut blocked = false;

    for _ in 0..max_iterations {
        let remaining_len = remaining.length();
        if remaining_len <= 1e-6 {
            break;
        }

        let mut best_hit_t = f32::INFINITY;
        let mut best_normal = Vec2::ZERO;
        let query_radius = collider.radius + remaining_len + skin + 0.1;

        grid.query_nearby(position, query_radius, |static_collider| {
            let feet_y = position.y - collider.half_height;
            let collider_top = static_collider.center.y + static_collider.half_extents.y;

            // Treat current support surfaces as floor, not as side blockers.
            // Without this, walking off an edge can get stuck on the same step's "wall".
            if feet_y >= collider_top - skin {
                return;
            }

            if !capsule_overlaps_aabb_vertically(
                position.y,
                collider,
                static_collider.center.y,
                static_collider.half_extents.y,
                skin * 0.25,
            ) {
                return;
            }

            if let Some((toi, normal)) = sweep_disc_against_aabb_xz(
                Vec2::new(position.x, position.z),
                remaining,
                collider.radius + skin,
                Vec2::new(static_collider.center.x, static_collider.center.z),
                Vec2::new(
                    static_collider.half_extents.x,
                    static_collider.half_extents.z,
                ),
            ) {
                if toi < best_hit_t {
                    best_hit_t = toi;
                    best_normal = normal;
                }
            }
        });

        if !best_hit_t.is_finite() {
            position.x += remaining.x;
            position.z += remaining.y;
            break;
        }

        blocked = true;
        let move_t = (best_hit_t - 0.001).clamp(0.0, 1.0);
        position.x += remaining.x * move_t;
        position.z += remaining.y * move_t;

        let mut leftover = remaining * (1.0 - best_hit_t.clamp(0.0, 1.0));
        let into_wall = leftover.dot(best_normal);
        if into_wall < 0.0 {
            leftover -= best_normal * into_wall;
        }
        remaining = leftover;
    }

    (position, blocked)
}

fn try_step_move(
    start: Vec3,
    displacement: Vec3,
    collider: PlayerCollider,
    grid: &WorldCollisionGrid,
    step_height: f32,
    max_drop: f32,
    skin: f32,
) -> Option<Vec3> {
    let horizontal_delta = Vec2::new(displacement.x, displacement.z);
    if horizontal_delta.length_squared() < 1e-6 {
        return None;
    }

    let raised = start + Vec3::Y * step_height;
    if would_collide(raised, collider, grid) {
        return None;
    }

    let (raised_moved, _) = move_with_slide(
        raised,
        displacement,
        collider,
        grid,
        CONTROLLER_MAX_SLIDES,
        skin,
    );

    let moved_dist = Vec2::new(raised_moved.x - start.x, raised_moved.z - start.z).length();
    if moved_dist < 0.02 {
        return None;
    }

    let current_top = start.y - collider.half_height;
    let mut best_step_up_top: Option<f32> = None;
    let mut best_flat_top: Option<f32> = None;
    grid.query_nearby(raised_moved, collider.radius + 0.1, |static_collider| {
        if !intersects_disc_aabb_xz(
            raised_moved,
            collider.radius,
            static_collider.center,
            static_collider.half_extents,
        ) {
            return;
        }

        let top = static_collider.center.y + static_collider.half_extents.y;
        let center_after_snap = top + collider.half_height;
        let drop = raised_moved.y - center_after_snap;
        if drop < -skin || drop > step_height + max_drop {
            return;
        }

        let step_up = top - current_top;
        if step_up > skin {
            best_step_up_top = Some(best_step_up_top.map_or(top, |current| current.min(top)));
        } else if step_up >= -skin {
            best_flat_top = Some(best_flat_top.map_or(top, |current| current.max(top)));
        }
    });

    let top = best_step_up_top.or(best_flat_top)?;
    let snapped = Vec3::new(raised_moved.x, top + collider.half_height, raised_moved.z);
    if would_collide(snapped, collider, grid) {
        return None;
    }

    Some(snapped)
}

fn capsule_overlaps_aabb_vertically(
    capsule_center_y: f32,
    capsule: PlayerCollider,
    box_center_y: f32,
    box_half_y: f32,
    extra_margin: f32,
) -> bool {
    let box_min = box_center_y - box_half_y;
    let box_max = box_center_y + box_half_y;
    let capsule_min = capsule_center_y - capsule.half_height;
    let capsule_max = capsule_center_y + capsule.half_height;

    capsule_min < box_max - extra_margin && capsule_max > box_min + extra_margin
}

fn sweep_disc_against_aabb_xz(
    origin: Vec2,
    delta: Vec2,
    radius: f32,
    box_center: Vec2,
    box_half: Vec2,
) -> Option<(f32, Vec2)> {
    let expanded_min = box_center - box_half - Vec2::splat(radius);
    let expanded_max = box_center + box_half + Vec2::splat(radius);

    if origin.x >= expanded_min.x
        && origin.x <= expanded_max.x
        && origin.y >= expanded_min.y
        && origin.y <= expanded_max.y
    {
        let left = (origin.x - expanded_min.x).abs();
        let right = (expanded_max.x - origin.x).abs();
        let down = (origin.y - expanded_min.y).abs();
        let up = (expanded_max.y - origin.y).abs();
        let min_side = left.min(right).min(down).min(up);

        let normal = if min_side == left {
            Vec2::new(-1.0, 0.0)
        } else if min_side == right {
            Vec2::new(1.0, 0.0)
        } else if min_side == down {
            Vec2::new(0.0, -1.0)
        } else {
            Vec2::new(0.0, 1.0)
        };
        return Some((0.0, normal));
    }

    let mut t_min: f32 = 0.0;
    let mut t_max: f32 = 1.0;
    let mut hit_normal = Vec2::ZERO;

    for axis in 0..2 {
        let (o, d, min_v, max_v) = if axis == 0 {
            (origin.x, delta.x, expanded_min.x, expanded_max.x)
        } else {
            (origin.y, delta.y, expanded_min.y, expanded_max.y)
        };

        if d.abs() <= 1e-8 {
            if o < min_v || o > max_v {
                return None;
            }
            continue;
        }

        let inv = 1.0 / d;
        let mut t1 = (min_v - o) * inv;
        let mut t2 = (max_v - o) * inv;
        let n = if axis == 0 {
            if d > 0.0 {
                Vec2::new(-1.0, 0.0)
            } else {
                Vec2::new(1.0, 0.0)
            }
        } else if d > 0.0 {
            Vec2::new(0.0, -1.0)
        } else {
            Vec2::new(0.0, 1.0)
        };

        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }

        if t1 > t_min {
            t_min = t1;
            hit_normal = n;
        }
        t_max = t_max.min(t2);
        if t_min > t_max {
            return None;
        }
    }

    if (0.0..=1.0).contains(&t_min) {
        Some((t_min, hit_normal))
    } else {
        None
    }
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

pub(super) fn billboard_stair_labels(
    camera_query: Query<&Transform, (With<Camera3d>, Without<StairSteepnessLabel>)>,
    mut labels: Query<&mut Transform, (With<StairSteepnessLabel>, Without<Camera3d>)>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    for mut label_transform in &mut labels {
        let to_camera = camera_transform.translation - label_transform.translation;
        let horizontal = Vec2::new(to_camera.x, to_camera.z);
        if horizontal.length_squared() < 1e-8 {
            continue;
        }

        let yaw = horizontal.x.atan2(horizontal.y);
        label_transform.rotation = Quat::from_rotation_y(yaw);
    }
}

pub(super) fn update_player_blob_shadow(
    settings: Res<GameSettings>,
    player_query: Query<(&Transform, &PlayerCollider), (With<Player>, Without<PlayerBlobShadow>)>,
    world_collision_grid: Res<WorldCollisionGrid>,
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
    let mut support_top: f32 = 0.0;
    world_collision_grid.query_nearby(player_pos, player_collider.radius + 1.0, |collider| {
        if !intersects_disc_aabb_xz(
            player_pos,
            player_collider.radius,
            collider.center,
            collider.half_extents,
        ) {
            return;
        }

        let top = collider.center.y + collider.half_extents.y;
        if top <= player_pos.y + 0.2 {
            support_top = support_top.max(top);
        }
    });

    let feet_height = player_pos.y - player_collider.half_height;
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

pub(super) fn draw_debug_geometry(
    debug: Res<DebugSettings>,
    player_query: Query<(&Transform, &PlayerCollider), With<Player>>,
    world_query: Query<(&Transform, &WorldCollider), Without<Player>>,
    mut gizmos: Gizmos,
) {
    if !debug.show_collision_shapes && !debug.show_world_axes {
        return;
    }

    if debug.show_world_axes {
        let len = 4.0;
        gizmos.line(
            Vec3::ZERO,
            Vec3::new(len, 0.0, 0.0),
            Color::srgb(0.9, 0.22, 0.22),
        );
        gizmos.line(
            Vec3::ZERO,
            Vec3::new(0.0, len, 0.0),
            Color::srgb(0.22, 0.9, 0.22),
        );
        gizmos.line(
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, len),
            Color::srgb(0.22, 0.52, 0.95),
        );
    }

    if !debug.show_collision_shapes {
        return;
    }

    for (transform, collider) in &world_query {
        draw_aabb_lines(
            &mut gizmos,
            transform.translation,
            collider.half_extents,
            Color::srgba(1.0, 0.9, 0.35, 0.95),
        );
    }

    if let Ok((transform, collider)) = player_query.single() {
        draw_capsule_lines(
            &mut gizmos,
            transform.translation,
            collider.radius,
            collider.half_height,
            Color::srgba(0.25, 1.0, 1.0, 0.95),
        );
    }
}

fn draw_aabb_lines(gizmos: &mut Gizmos, center: Vec3, half: Vec3, color: Color) {
    let min = center - half;
    let max = center + half;

    let p000 = Vec3::new(min.x, min.y, min.z);
    let p001 = Vec3::new(min.x, min.y, max.z);
    let p010 = Vec3::new(min.x, max.y, min.z);
    let p011 = Vec3::new(min.x, max.y, max.z);
    let p100 = Vec3::new(max.x, min.y, min.z);
    let p101 = Vec3::new(max.x, min.y, max.z);
    let p110 = Vec3::new(max.x, max.y, min.z);
    let p111 = Vec3::new(max.x, max.y, max.z);

    gizmos.line(p000, p001, color);
    gizmos.line(p000, p010, color);
    gizmos.line(p000, p100, color);
    gizmos.line(p001, p011, color);
    gizmos.line(p001, p101, color);
    gizmos.line(p010, p011, color);
    gizmos.line(p010, p110, color);
    gizmos.line(p100, p101, color);
    gizmos.line(p100, p110, color);
    gizmos.line(p111, p101, color);
    gizmos.line(p111, p110, color);
    gizmos.line(p111, p011, color);
}

fn draw_capsule_lines(
    gizmos: &mut Gizmos,
    center: Vec3,
    radius: f32,
    half_height: f32,
    color: Color,
) {
    let ring_segments = 20;
    let cyl_half = (half_height - radius).max(0.0);
    let top_y = center.y + cyl_half;
    let bottom_y = center.y - cyl_half;
    let vertical_top = center.y + half_height;
    let vertical_bottom = center.y - half_height;

    draw_ring(
        gizmos,
        Vec3::new(center.x, top_y, center.z),
        radius,
        color,
        ring_segments,
    );
    draw_ring(
        gizmos,
        Vec3::new(center.x, bottom_y, center.z),
        radius,
        color,
        ring_segments,
    );

    let cardinal = [
        Vec3::new(radius, 0.0, 0.0),
        Vec3::new(-radius, 0.0, 0.0),
        Vec3::new(0.0, 0.0, radius),
        Vec3::new(0.0, 0.0, -radius),
    ];
    for offset in cardinal {
        gizmos.line(
            Vec3::new(center.x + offset.x, top_y, center.z + offset.z),
            Vec3::new(center.x + offset.x, bottom_y, center.z + offset.z),
            color,
        );
        gizmos.line(
            Vec3::new(center.x + offset.x, top_y, center.z + offset.z),
            Vec3::new(center.x, vertical_top, center.z),
            color,
        );
        gizmos.line(
            Vec3::new(center.x + offset.x, bottom_y, center.z + offset.z),
            Vec3::new(center.x, vertical_bottom, center.z),
            color,
        );
    }
}

fn draw_ring(gizmos: &mut Gizmos, center: Vec3, radius: f32, color: Color, segments: usize) {
    if segments < 3 {
        return;
    }

    let mut prev = center + Vec3::new(radius, 0.0, 0.0);
    for i in 1..=segments {
        let t = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let point = center + Vec3::new(radius * t.cos(), 0.0, radius * t.sin());
        gizmos.line(prev, point, color);
        prev = point;
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
    player_collider: PlayerCollider,
    world_collision_grid: &WorldCollisionGrid,
) -> bool {
    let mut hit = false;
    world_collision_grid.query_nearby(player_center, player_collider.radius + 0.1, |collider| {
        if hit {
            return;
        }

        hit = intersects_vertical_capsule_aabb(
            player_center,
            player_collider.radius,
            player_collider.half_height,
            collider.center,
            collider.half_extents,
        );
    });
    hit
}

pub(super) fn find_landing_top(
    previous_center: Vec3,
    proposed_center: Vec3,
    player_collider: PlayerCollider,
    world_collision_grid: &WorldCollisionGrid,
) -> Option<f32> {
    let previous_bottom = previous_center.y - player_collider.half_height;
    let proposed_bottom = proposed_center.y - player_collider.half_height;
    let epsilon = 0.0001;
    let mut top_hit: Option<f32> = None;

    world_collision_grid.query_nearby(proposed_center, player_collider.radius + 0.1, |collider| {
        if !intersects_disc_aabb_xz(
            proposed_center,
            player_collider.radius,
            collider.center,
            collider.half_extents,
        ) {
            return;
        }

        let collider_top = collider.center.y + collider.half_extents.y;
        let crossed_top =
            previous_bottom >= collider_top - epsilon && proposed_bottom <= collider_top + epsilon;

        if crossed_top {
            top_hit = Some(top_hit.map_or(collider_top, |best| best.max(collider_top)));
        }
    });

    top_hit
}

pub(super) fn find_ceiling_bottom(
    previous_center: Vec3,
    proposed_center: Vec3,
    player_collider: PlayerCollider,
    world_collision_grid: &WorldCollisionGrid,
) -> Option<f32> {
    let previous_top = previous_center.y + player_collider.half_height;
    let proposed_top = proposed_center.y + player_collider.half_height;
    let epsilon = 0.0001;
    let mut bottom_hit: Option<f32> = None;

    world_collision_grid.query_nearby(proposed_center, player_collider.radius + 0.1, |collider| {
        if !intersects_disc_aabb_xz(
            proposed_center,
            player_collider.radius,
            collider.center,
            collider.half_extents,
        ) {
            return;
        }

        let collider_bottom = collider.center.y - collider.half_extents.y;
        let crossed_bottom =
            previous_top <= collider_bottom + epsilon && proposed_top >= collider_bottom - epsilon;

        if crossed_bottom {
            bottom_hit = Some(bottom_hit.map_or(collider_bottom, |best| best.min(collider_bottom)));
        }
    });

    bottom_hit
}

pub(super) fn intersects_disc_aabb_xz(
    disc_center: Vec3,
    disc_radius: f32,
    box_center: Vec3,
    box_half_extents: Vec3,
) -> bool {
    let dx = (disc_center.x - box_center.x).abs() - box_half_extents.x;
    let dz = (disc_center.z - box_center.z).abs() - box_half_extents.z;
    let outside_x = dx.max(0.0);
    let outside_z = dz.max(0.0);
    let dist_sq = outside_x * outside_x + outside_z * outside_z;
    let radius_sq = disc_radius * disc_radius;
    dist_sq <= radius_sq + 1e-5
}

pub(super) fn intersects_vertical_capsule_aabb(
    capsule_center: Vec3,
    capsule_radius: f32,
    capsule_half_height: f32,
    box_center: Vec3,
    box_half_extents: Vec3,
) -> bool {
    let dx = (capsule_center.x - box_center.x).abs() - box_half_extents.x;
    let dz = (capsule_center.z - box_center.z).abs() - box_half_extents.z;
    let outside_x = dx.max(0.0);
    let outside_z = dz.max(0.0);

    let capsule_seg_min = capsule_center.y - (capsule_half_height - capsule_radius).max(0.0);
    let capsule_seg_max = capsule_center.y + (capsule_half_height - capsule_radius).max(0.0);
    let box_min = box_center.y - box_half_extents.y;
    let box_max = box_center.y + box_half_extents.y;

    let outside_y = if capsule_seg_max < box_min {
        box_min - capsule_seg_max
    } else if capsule_seg_min > box_max {
        capsule_seg_min - box_max
    } else {
        0.0
    };

    outside_x * outside_x + outside_y * outside_y + outside_z * outside_z
        < capsule_radius * capsule_radius
}
