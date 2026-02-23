use super::*;

pub(super) fn setup_start_menu(
    mut commands: Commands,
    flow: Res<GameFlowState>,
    scenarios: Res<ScenarioCatalog>,
) {
    if flow.pending_scenario.is_none() {
        commands.spawn((Camera2d, StartMenuCamera));
        spawn_start_menu_ui(&mut commands, &scenarios);
    }
}

pub(super) fn spawn_start_menu_ui(commands: &mut Commands, scenarios: &ScenarioCatalog) {
    commands
        .spawn((
            StartMenuRoot,
            GlobalZIndex(700),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.94)),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: px(620),
                    padding: UiRect::all(px(22)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.11, 0.13, 0.17)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("Selecteer Scenario"),
                    Node {
                        margin: UiRect::bottom(px(12)),
                        ..default()
                    },
                ));

                for (index, scenario) in scenarios.scenarios.iter().enumerate() {
                    panel.spawn((
                        Text::new(format!("{}: {}", scenario.name, scenario.description)),
                        Node {
                            margin: UiRect::bottom(px(6)),
                            ..default()
                        },
                    ));

                    panel
                        .spawn((
                            Button,
                            StartMenuButton(StartMenuButtonAction::StartScenario(index)),
                            menu_button_node(),
                            menu_button_normal_color(),
                        ))
                        .with_child(Text::new(format!("Start {}", scenario.name)));
                }

                panel
                    .spawn((
                        Button,
                        StartMenuButton(StartMenuButtonAction::ExitGame),
                        menu_button_node(),
                        menu_button_normal_color(),
                    ))
                    .with_child(Text::new("Exit"));
            });
        });
}

pub(super) fn handle_start_menu_buttons(
    mut interactions: Query<
        (&Interaction, &StartMenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut flow: ResMut<GameFlowState>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if flow.in_game {
        return;
    }

    for (interaction, button, mut background) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *background = menu_button_pressed_color();
                match button.0 {
                    StartMenuButtonAction::StartScenario(scenario) => {
                        flow.pending_scenario = Some(scenario);
                    }
                    StartMenuButtonAction::ExitGame => {
                        app_exit.write(AppExit::Success);
                    }
                }
            }
            Interaction::Hovered => {
                *background = menu_button_hover_color();
            }
            Interaction::None => {
                *background = menu_button_normal_color();
            }
        }
    }
}

pub(super) fn load_pending_scenario(
    mut commands: Commands,
    mut flow: ResMut<GameFlowState>,
    scenarios: Res<ScenarioCatalog>,
    asset_server: Res<AssetServer>,
    mut settings: ResMut<GameSettings>,
    mut menu: ResMut<MenuState>,
    start_menu_roots: Query<Entity, With<StartMenuRoot>>,
    start_menu_cameras: Query<Entity, With<StartMenuCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(scenario_index) = flow.pending_scenario.take() else {
        return;
    };
    let Some(scenario) = scenarios.scenarios.get(scenario_index).cloned() else {
        eprintln!("Scenario index {} is ongeldig", scenario_index);
        return;
    };

    for root in &start_menu_roots {
        commands.entity(root).despawn();
    }
    for camera in &start_menu_cameras {
        commands.entity(camera).despawn();
    }

    menu.open = false;
    menu.screen = MenuScreen::Main;
    menu.awaiting_rebind = None;
    menu.dirty = false;

    spawn_scenario_world(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut materials,
        &scenario,
    );
    settings.set_changed();
    flow.in_game = true;
}

fn default_distance_fog() -> DistanceFog {
    DistanceFog {
        color: Color::srgba(0.62, 0.72, 0.84, 1.0),
        directional_light_color: Color::srgba(0.97, 0.88, 0.70, 0.5),
        directional_light_exponent: 20.0,
        falloff: FogFalloff::Linear {
            start: 22.0,
            end: 78.0,
        },
    }
}

pub(super) fn spawn_scenario_world(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    scenario: &ScenarioDefinition,
) {
    let ground_extent = scenario.ground_extent;
    let crate_grid_radius = scenario.crate_grid_radius;
    let crate_spacing = scenario.crate_spacing;
    let crate_pattern_mod = scenario.crate_pattern_mod.max(1);
    let wall_count = scenario.wall_count;
    let wall_spacing = scenario.wall_spacing;
    let wall_z = scenario.wall_z;
    let tower_z = scenario.tower_z;
    let sun_position = scenario.sun_vec3();
    let mut static_colliders = Vec::new();

    let player_radius: f32 = 0.35;
    let player_half_height: f32 = 0.9;
    let player_cylinder_length = (player_half_height * 2.0 - player_radius * 2.0).max(0.0_f32);
    let player_mesh = meshes.add(Capsule3d::new(player_radius, player_cylinder_length));
    let player_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.91, 0.84, 0.64),
        perceptual_roughness: 0.88,
        ..default()
    });
    let baked_shadow_mesh = meshes.add(Plane3d::default().mesh().size(1.0, 1.0));
    let baked_shadow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.34),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let player_shadow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.58),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        Player::default(),
        Mesh3d(player_mesh),
        MeshMaterial3d(player_mat),
        Transform::from_xyz(0.0, player_half_height, 0.0),
        NotShadowCaster,
        PlayerCollider {
            radius: player_radius,
            half_height: player_half_height,
        },
        PlayerKinematics {
            vertical_velocity: 0.0,
            grounded: true,
        },
        InGameEntity,
    ));

    commands.spawn((
        PlayerBlobShadow,
        Mesh3d(baked_shadow_mesh.clone()),
        MeshMaterial3d(player_shadow_mat),
        Transform::from_xyz(0.0, 0.015, 0.0).with_scale(Vec3::new(0.9, 1.0, 0.9)),
        NotShadowCaster,
        NotShadowReceiver,
        InGameEntity,
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 4.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ThirdPersonCameraRig::default(),
        Msaa::Sample4,
        default_distance_fog(),
        InGameEntity,
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.90, 0.70),
            shadows_enabled: true,
            illuminance: 12_500.0,
            ..default()
        },
        Transform::from_translation(sun_position).looking_at(Vec3::ZERO, Vec3::Y),
        InGameEntity,
    ));

    let ground_mesh = meshes.add(Cuboid::new(ground_extent, 0.1, ground_extent));
    let ground_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.43, 0.20),
        perceptual_roughness: 1.0,
        ..default()
    });

    let ground_center = Vec3::new(0.0, -0.05, 0.0);
    let ground_half = Vec3::new(ground_extent * 0.5, 0.05, ground_extent * 0.5);
    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(ground_mat),
        Transform::from_translation(ground_center),
        WorldCollider {
            half_extents: ground_half,
        },
        InGameEntity,
    ));
    static_colliders.push(StaticCollider {
        center: ground_center,
        half_extents: ground_half,
    });

    let wall_mesh = meshes.add(Cuboid::new(3.0, 3.0, 3.0));
    let tower_mesh = meshes.add(Cuboid::new(4.0, 8.0, 4.0));
    let crate_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.57, 0.52, 0.44),
        perceptual_roughness: 0.92,
        ..default()
    });
    let tower_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.34, 0.38, 0.55),
        metallic: 0.0,
        perceptual_roughness: 0.88,
        ..default()
    });
    let crate_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.69, 0.44, 0.24),
        perceptual_roughness: 0.86,
        ..default()
    });

    for x in -crate_grid_radius..=crate_grid_radius {
        for z in -crate_grid_radius..=crate_grid_radius {
            let near_spawn = (-1..=1).contains(&x) && (-1..=1).contains(&z);
            if (x + z).rem_euclid(crate_pattern_mod) == 0 && !near_spawn {
                commands.spawn((
                    Mesh3d(crate_mesh.clone()),
                    MeshMaterial3d(crate_mat.clone()),
                    Transform::from_xyz(x as f32 * crate_spacing, 0.5, z as f32 * crate_spacing),
                    NotShadowCaster,
                    WorldCollider {
                        half_extents: Vec3::splat(0.5),
                    },
                    InGameEntity,
                ));
                static_colliders.push(StaticCollider {
                    center: Vec3::new(x as f32 * crate_spacing, 0.5, z as f32 * crate_spacing),
                    half_extents: Vec3::splat(0.5),
                });
                spawn_baked_shadow(
                    commands,
                    &baked_shadow_mesh,
                    &baked_shadow_mat,
                    Vec3::new(x as f32 * crate_spacing, 0.011, z as f32 * crate_spacing),
                    Vec2::new(1.25, 1.25),
                );
            }
        }
    }

    for i in -wall_count..=wall_count {
        let wall_center = Vec3::new(i as f32 * wall_spacing, 1.5, wall_z);
        commands.spawn((
            Mesh3d(wall_mesh.clone()),
            MeshMaterial3d(wall_mat.clone()),
            Transform::from_translation(wall_center),
            NotShadowCaster,
            WorldCollider {
                half_extents: Vec3::splat(1.5),
            },
            InGameEntity,
        ));
        static_colliders.push(StaticCollider {
            center: wall_center,
            half_extents: Vec3::splat(1.5),
        });
        spawn_baked_shadow(
            commands,
            &baked_shadow_mesh,
            &baked_shadow_mat,
            Vec3::new(i as f32 * wall_spacing, 0.011, wall_z),
            Vec2::new(3.4, 3.0),
        );
    }

    let tower_center = Vec3::new(0.0, 4.0, tower_z);
    let tower_half = Vec3::new(2.0, 4.0, 2.0);
    commands.spawn((
        Mesh3d(tower_mesh),
        MeshMaterial3d(tower_mat),
        Transform::from_translation(tower_center),
        NotShadowCaster,
        WorldCollider {
            half_extents: tower_half,
        },
        InGameEntity,
    ));
    static_colliders.push(StaticCollider {
        center: tower_center,
        half_extents: tower_half,
    });
    spawn_baked_shadow(
        commands,
        &baked_shadow_mesh,
        &baked_shadow_mat,
        Vec3::new(0.0, 0.011, tower_z),
        Vec2::new(5.0, 5.0),
    );

    if scenario.id == "greenwood" {
        // Place the generated table model as a scene in Greenwood Valley.
        let table_origin = Vec3::new(7.0, 0.0, -5.0);
        commands.spawn((
            SceneRoot(asset_server.load("models/table.glb#Scene0")),
            Transform::from_translation(table_origin),
            InGameEntity,
        ));

        // Single AABB around the current table export (derived from live_report.json).
        // If the model shape changes, update this center + half extents.
        let table_collider_center = table_origin + Vec3::new(0.0, 0.2053, 0.0);
        let table_collider_half = Vec3::new(1.0, 0.2053, 0.6);
        commands.spawn((
            Transform::from_translation(table_collider_center),
            WorldCollider {
                half_extents: table_collider_half,
            },
            InGameEntity,
        ));
        static_colliders.push(StaticCollider {
            center: table_collider_center,
            half_extents: table_collider_half,
        });

        // Add 5 stair variants with different steepness for controller testing.
        // (rise/run): from shallow to steep.
        let stair_profiles = [
            (0.16_f32, 1.05_f32),
            (0.20_f32, 0.92_f32),
            (0.24_f32, 0.80_f32),
            (0.30_f32, 0.72_f32),
            (0.36_f32, 0.64_f32),
        ];
        let stair_width = 2.2_f32;
        let stair_depth = 0.82_f32;
        let stairs_per_profile = 5;
        let base_x = -18.0_f32;
        let lane_spacing = 4.5_f32;
        let base_z = 8.0_f32;
        let stair_colors = [
            Color::srgb(0.52, 0.58, 0.62),
            Color::srgb(0.56, 0.57, 0.49),
            Color::srgb(0.58, 0.52, 0.48),
            Color::srgb(0.52, 0.50, 0.58),
            Color::srgb(0.60, 0.50, 0.42),
        ];

        for (lane_idx, (stair_rise, stair_run)) in stair_profiles.into_iter().enumerate() {
            let lane_x = base_x + lane_spacing * lane_idx as f32;
            let stair_mesh = meshes.add(Cuboid::new(stair_width, stair_rise, stair_depth));
            let stair_mat = materials.add(StandardMaterial {
                base_color: stair_colors[lane_idx % stair_colors.len()],
                perceptual_roughness: 0.94,
                ..default()
            });

            commands.spawn((
                StairSteepnessLabel,
                InGameEntity,
                Text2d::new(format!("rise {:.2}\nrun {:.2}", stair_rise, stair_run)),
                TextLayout::new_with_justify(Justify::Center),
                TextFont::from_font_size(26.0),
                TextColor(Color::srgb(0.96, 0.96, 0.94)),
                TextBackgroundColor(Color::srgba(0.08, 0.10, 0.12, 0.68)),
                Transform::from_xyz(lane_x, 0.38, base_z - 0.55).with_scale(Vec3::splat(0.018)),
            ));

            for step in 0..stairs_per_profile {
                let idx = step as f32;
                let center = Vec3::new(
                    lane_x,
                    stair_rise * 0.5 + idx * stair_rise,
                    base_z + idx * stair_run + stair_depth * 0.5,
                );
                let half = Vec3::new(stair_width * 0.5, stair_rise * 0.5, stair_depth * 0.5);

                commands.spawn((
                    Mesh3d(stair_mesh.clone()),
                    MeshMaterial3d(stair_mat.clone()),
                    Transform::from_translation(center),
                    NotShadowCaster,
                    WorldCollider { half_extents: half },
                    InGameEntity,
                ));
                static_colliders.push(StaticCollider {
                    center,
                    half_extents: half,
                });
                spawn_baked_shadow(
                    commands,
                    &baked_shadow_mesh,
                    &baked_shadow_mat,
                    Vec3::new(center.x, 0.011, center.z),
                    Vec2::new(stair_width * 1.05, stair_depth * 1.15),
                );
            }
        }
    }

    commands.insert_resource(WorldCollisionGrid::from_colliders(static_colliders, 4.0));

    commands
        .spawn((
            InGameEntity,
            Node {
                position_type: PositionType::Absolute,
                top: px(12),
                left: px(12),
                ..default()
            },
        ))
        .with_child(Text::new(
            format!(
                "Scenario: {}\nESC: menu\nLMB: camera orbit\nRMB: aim-move mode\nScroll: zoom\n\nKeybinds zijn aanpasbaar in het menu.",
                scenario.name
            ),
        ));

    commands.spawn((
        PerformanceOverlayText,
        InGameEntity,
        Text::new("FPS: --\nFrame time: -- ms"),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            right: px(12),
            ..default()
        },
    ));
}

pub(super) fn toggle_menu_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    flow: Res<GameFlowState>,
    mut menu: ResMut<MenuState>,
) {
    if !flow.in_game {
        return;
    }

    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    if menu.open {
        menu.open = false;
        menu.screen = MenuScreen::Main;
        menu.awaiting_rebind = None;
    } else {
        menu.open = true;
        menu.screen = MenuScreen::Main;
        menu.awaiting_rebind = None;
    }

    menu.dirty = true;
}

pub(super) fn handle_menu_buttons(
    mut interactions: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut commands: Commands,
    scenarios: Res<ScenarioCatalog>,
    mut flow: ResMut<GameFlowState>,
    mut menu: ResMut<MenuState>,
    mut settings: ResMut<GameSettings>,
    mut debug: ResMut<DebugSettings>,
    keybinds: ResMut<GameKeybinds>,
    in_game_entities: Query<Entity, With<InGameEntity>>,
    start_menu_roots: Query<Entity, With<StartMenuRoot>>,
    start_menu_cameras: Query<Entity, With<StartMenuCamera>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if !menu.open {
        return;
    }

    let mut should_save = false;

    for (interaction, menu_button, mut background) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *background = menu_button_pressed_color();

                match menu_button.0 {
                    MenuButtonAction::Resume => {
                        menu.open = false;
                        menu.screen = MenuScreen::Main;
                        menu.awaiting_rebind = None;
                    }
                    MenuButtonAction::OpenSettings => {
                        menu.screen = MenuScreen::Settings;
                        menu.awaiting_rebind = None;
                    }
                    MenuButtonAction::OpenDebug => {
                        menu.screen = MenuScreen::Debug;
                        menu.awaiting_rebind = None;
                    }
                    MenuButtonAction::OpenKeybinds => {
                        menu.screen = MenuScreen::Keybinds;
                        menu.awaiting_rebind = None;
                    }
                    MenuButtonAction::OpenExitConfirm => {
                        menu.screen = MenuScreen::ExitConfirm;
                        menu.awaiting_rebind = None;
                    }
                    MenuButtonAction::BackMain => {
                        menu.screen = MenuScreen::Main;
                        menu.awaiting_rebind = None;
                    }
                    MenuButtonAction::ExitNow => {
                        if flow.in_game {
                            for entity in &in_game_entities {
                                commands.entity(entity).despawn();
                            }
                            for root in &start_menu_roots {
                                commands.entity(root).despawn();
                            }
                            for camera in &start_menu_cameras {
                                commands.entity(camera).despawn();
                            }

                            commands.spawn((Camera2d, StartMenuCamera));
                            spawn_start_menu_ui(&mut commands, &scenarios);

                            flow.in_game = false;
                            flow.pending_scenario = None;

                            menu.open = false;
                            menu.screen = MenuScreen::Main;
                            menu.awaiting_rebind = None;
                            menu.keybind_filter.clear();
                        } else {
                            app_exit.write(AppExit::Success);
                        }
                    }
                    MenuButtonAction::CycleDisplayMode => {
                        settings.display_mode = settings.display_mode.next();
                        should_save = true;
                    }
                    MenuButtonAction::CycleResolution => {
                        let current = (settings.resolution_width, settings.resolution_height);
                        let next_idx = RESOLUTION_OPTIONS
                            .iter()
                            .position(|&res| res == current)
                            .map(|idx| (idx + 1) % RESOLUTION_OPTIONS.len())
                            .unwrap_or(0);
                        let next = RESOLUTION_OPTIONS[next_idx];
                        settings.resolution_width = next.0;
                        settings.resolution_height = next.1;
                        should_save = true;
                    }
                    MenuButtonAction::ToggleMsaa => {
                        settings.msaa_enabled = !settings.msaa_enabled;
                        should_save = true;
                    }
                    MenuButtonAction::ToggleShadowMode => {
                        settings.shadow_mode = settings.shadow_mode.next();
                        should_save = true;
                    }
                    MenuButtonAction::TogglePerformanceOverlay => {
                        debug.show_performance_overlay = !debug.show_performance_overlay;
                        should_save = true;
                    }
                    MenuButtonAction::ToggleBakedShadows => {
                        debug.show_baked_shadows = !debug.show_baked_shadows;
                        should_save = true;
                    }
                    MenuButtonAction::ToggleFog => {
                        debug.show_fog = !debug.show_fog;
                        should_save = true;
                    }
                    MenuButtonAction::ToggleCollisionShapes => {
                        debug.show_collision_shapes = !debug.show_collision_shapes;
                        should_save = true;
                    }
                    MenuButtonAction::ToggleWireframe => {
                        debug.show_wireframe = !debug.show_wireframe;
                        should_save = true;
                    }
                    MenuButtonAction::ToggleWorldAxes => {
                        debug.show_world_axes = !debug.show_world_axes;
                        should_save = true;
                    }
                    MenuButtonAction::StartRebind(action) => {
                        menu.screen = MenuScreen::Keybinds;
                        menu.awaiting_rebind = Some(action);
                    }
                    MenuButtonAction::ClearKeybindFilter => {
                        menu.keybind_filter.clear();
                    }
                }

                menu.dirty = true;
            }
            Interaction::Hovered => {
                *background = menu_button_hover_color();
            }
            Interaction::None => {
                *background = menu_button_normal_color();
            }
        }
    }

    if should_save {
        save_persisted_config(&settings, &keybinds, &debug);
    }
}

pub(super) fn capture_rebind_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<MenuState>,
    settings: Res<GameSettings>,
    debug: Res<DebugSettings>,
    mut keybinds: ResMut<GameKeybinds>,
) {
    if !menu.open || menu.screen != MenuScreen::Keybinds {
        return;
    }

    let Some(action) = menu.awaiting_rebind else {
        return;
    };

    for key in keys.get_just_pressed() {
        if *key == KeyCode::Escape {
            continue;
        }

        let changed = if keybinds.has_key(action, *key) {
            keybinds.remove_key(action, *key)
        } else {
            keybinds.add_key(action, *key)
        };
        menu.awaiting_rebind = None;
        if changed {
            save_persisted_config(&settings, &keybinds, &debug);
        }
        menu.dirty = true;
        break;
    }
}

pub(super) fn capture_keybind_filter_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<MenuState>,
) {
    if !menu.open || menu.screen != MenuScreen::Keybinds || menu.awaiting_rebind.is_some() {
        return;
    }

    let mut changed = false;

    if keys.just_pressed(KeyCode::Backspace) {
        if menu.keybind_filter.pop().is_some() {
            changed = true;
        }
    }

    for key in keys.get_just_pressed() {
        if let Some(ch) = keycode_to_filter_char(*key) {
            menu.keybind_filter.push(ch);
            changed = true;
        }
    }

    if changed {
        menu.dirty = true;
    }
}

pub(super) fn rebuild_menu_ui(
    mut commands: Commands,
    flow: Res<GameFlowState>,
    mut menu: ResMut<MenuState>,
    existing_roots: Query<Entity, With<MenuRoot>>,
    settings: Res<GameSettings>,
    debug: Res<DebugSettings>,
    keybinds: Res<GameKeybinds>,
) {
    if !menu.dirty {
        return;
    }

    for entity in &existing_roots {
        commands.entity(entity).despawn();
    }

    if !menu.open {
        menu.dirty = false;
        return;
    }

    commands
        .spawn((
            MenuRoot,
            GlobalZIndex(500),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: px(520),
                    padding: UiRect::all(px(18)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.14, 0.18)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(match menu.screen {
                        MenuScreen::Main => "Game Menu",
                        MenuScreen::Settings => "Settings",
                        MenuScreen::Debug => "Debug",
                        MenuScreen::Keybinds => "Keybinds",
                        MenuScreen::ExitConfirm => {
                            if flow.in_game {
                                "Terug naar hoofdmenu"
                            } else {
                                "Exit"
                            }
                        }
                    }),
                    Node {
                        margin: UiRect::bottom(px(12)),
                        ..default()
                    },
                ));

                match menu.screen {
                    MenuScreen::Main => {
                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::Resume),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new("Resume"));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::OpenSettings),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new("Settings"));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::OpenKeybinds),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new("Keybinds"));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::OpenDebug),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new("Debug"));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::OpenExitConfirm),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new(if flow.in_game {
                                "Terug naar hoofdmenu"
                            } else {
                                "Exit"
                            }));
                    }
                    MenuScreen::Settings => {
                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::CycleDisplayMode),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new(format!(
                                "Display mode: {}",
                                settings.display_mode.label()
                            )));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::CycleResolution),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new(format!(
                                "Resolution: {}x{}",
                                settings.resolution_width, settings.resolution_height
                            )));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::ToggleMsaa),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new(format!(
                                "MSAA: {}",
                                if settings.msaa_enabled { "On" } else { "Off" }
                            )));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::ToggleShadowMode),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new(format!(
                                "Player Shadow: {}",
                                settings.shadow_mode.label()
                            )));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::BackMain),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new("Back"));
                    }
                    MenuScreen::Debug => {
                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::TogglePerformanceOverlay),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new(format!(
                                "Performance Overlay: {}",
                                if debug.show_performance_overlay {
                                    "On"
                                } else {
                                    "Off"
                                }
                            )));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::ToggleBakedShadows),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new(format!(
                                "World Baked Shadows: {}",
                                if debug.show_baked_shadows {
                                    "On"
                                } else {
                                    "Off"
                                }
                            )));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::ToggleFog),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new(format!(
                                "Fog: {}",
                                if debug.show_fog { "On" } else { "Off" }
                            )));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::ToggleCollisionShapes),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new(format!(
                                "Collision Shapes: {}",
                                if debug.show_collision_shapes {
                                    "On"
                                } else {
                                    "Off"
                                }
                            )));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::ToggleWireframe),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new(format!(
                                "Model Lines (Wireframe): {}",
                                if debug.show_wireframe { "On" } else { "Off" }
                            )));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::ToggleWorldAxes),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new(format!(
                                "World Axes: {}",
                                if debug.show_world_axes { "On" } else { "Off" }
                            )));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::BackMain),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new("Back"));
                    }
                    MenuScreen::Keybinds => {
                        panel.spawn((
                            Text::new(format!(
                                "Filter functions: {}",
                                if menu.keybind_filter.is_empty() {
                                    "<none>".to_string()
                                } else {
                                    menu.keybind_filter.clone()
                                }
                            )),
                            Node {
                                margin: UiRect::bottom(px(8)),
                                ..default()
                            },
                        ));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::ClearKeybindFilter),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new("Clear filter"));

                        if let Some(action) = menu.awaiting_rebind {
                            panel.spawn((
                                Text::new(format!(
                                    "Press a key for {} (toggle bind, ESC is reserved)",
                                    action.label()
                                )),
                                Node {
                                    margin: UiRect::bottom(px(8)),
                                    ..default()
                                },
                            ));
                        } else {
                            panel.spawn((
                                Text::new("Type to filter by function name. Backspace removes characters."),
                                Node {
                                    margin: UiRect::bottom(px(8)),
                                    ..default()
                                },
                            ));
                        }

                        for action in ACTION_ORDER {
                            if !action_matches_filter(action, &menu.keybind_filter) {
                                continue;
                            }

                            let key_name = keybinds.display_keys(action);
                            let label = if menu.awaiting_rebind == Some(action) {
                                format!("{}: <waiting>", action.label())
                            } else {
                                format!("{}: {}", action.label(), key_name)
                            };

                            panel
                                .spawn((
                                    Button,
                                    MenuButton(MenuButtonAction::StartRebind(action)),
                                    menu_button_node(),
                                    menu_button_normal_color(),
                                ))
                                .with_child(Text::new(label));
                        }

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::BackMain),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new("Back"));
                    }
                    MenuScreen::ExitConfirm => {
                        panel.spawn((
                            Text::new(if flow.in_game {
                                "Terug naar het hoofdmenu?"
                            } else {
                                "Weet je het zeker?"
                            }),
                            Node {
                                margin: UiRect::bottom(px(10)),
                                ..default()
                            },
                        ));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::ExitNow),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new(if flow.in_game {
                                "Ja, hoofdmenu"
                            } else {
                                "Ja, Exit"
                            }));

                        panel
                            .spawn((
                                Button,
                                MenuButton(MenuButtonAction::BackMain),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new("Nee, terug"));
                    }
                }
            });
        });

    menu.dirty = false;
}

pub(super) fn apply_runtime_settings(
    settings: Res<GameSettings>,
    debug: Res<DebugSettings>,
    primary_window: Single<&mut Window, With<PrimaryWindow>>,
    camera_entities: Query<Entity, With<Camera3d>>,
    player_entities: Query<(Entity, Has<NotShadowCaster>), With<Player>>,
    mut lights: Query<&mut DirectionalLight>,
    mut visibility_queries: ParamSet<(
        Query<&mut Visibility, (With<PlayerBlobShadow>, Without<BakedShadow>)>,
        Query<&mut Visibility, (With<BakedShadow>, Without<PlayerBlobShadow>)>,
        Query<&mut Visibility, With<PerformanceOverlayText>>,
    )>,
    mut render_mode_queries: ParamSet<(
        Query<
            Entity,
            (
                With<Mesh3d>,
                Without<Wireframe>,
                Without<PlayerBlobShadow>,
                Without<BakedShadow>,
            ),
        >,
        Query<
            Entity,
            (
                With<Mesh3d>,
                With<Wireframe>,
                Without<PlayerBlobShadow>,
                Without<BakedShadow>,
            ),
        >,
    )>,
    camera_has_fog: Query<(), (With<Camera3d>, With<DistanceFog>)>,
    mut commands: Commands,
) {
    if !settings.is_changed() && !debug.is_changed() {
        return;
    }

    if settings.is_changed() {
        let mut window = primary_window.into_inner();
        window.mode = settings.display_mode.to_window_mode();
        window.resolution.set(
            settings.resolution_width as f32,
            settings.resolution_height as f32,
        );
    }

    if let Ok(camera) = camera_entities.single() {
        if settings.msaa_enabled {
            commands.entity(camera).insert(Msaa::Sample4);
        } else {
            commands.entity(camera).insert(Msaa::Off);
        }

        let has_fog = camera_has_fog.get(camera).is_ok();
        if debug.show_fog {
            if !has_fog {
                commands.entity(camera).insert(default_distance_fog());
            }
        } else if has_fog {
            commands.entity(camera).remove::<DistanceFog>();
        }
    }

    let stencil_mode = settings.shadow_mode == ShadowModeSetting::Stencil;

    for mut light in &mut lights {
        light.shadows_enabled = stencil_mode;
    }

    for mut visibility in &mut visibility_queries.p0() {
        *visibility = if stencil_mode {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }

    for mut visibility in &mut visibility_queries.p1() {
        *visibility = if !debug.show_baked_shadows {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }

    if let Ok((player, has_not_shadow_caster)) = player_entities.single() {
        if stencil_mode {
            if has_not_shadow_caster {
                commands.entity(player).remove::<NotShadowCaster>();
            }
        } else if !has_not_shadow_caster {
            commands.entity(player).insert(NotShadowCaster);
        }
    }

    for mut visibility in &mut visibility_queries.p2() {
        *visibility = if debug.show_performance_overlay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if debug.show_wireframe {
        for entity in &mut render_mode_queries.p0() {
            commands.entity(entity).insert(Wireframe);
        }
    } else {
        for entity in &mut render_mode_queries.p1() {
            commands.entity(entity).remove::<Wireframe>();
        }
    }
}
