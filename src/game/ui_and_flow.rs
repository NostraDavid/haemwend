use super::*;
use bevy_egui::{EguiContexts, PrimaryEguiContext, egui};

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
    mut images: ResMut<Assets<Image>>,
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
        &mut images,
        &scenario,
    );
    settings.set_changed();
    flow.in_game = true;
}

fn default_distance_fog() -> DistanceFog {
    distance_fog_from_debug(&DebugSettings::default(), 0.0)
}

#[derive(Clone, Copy)]
enum FogPreset {
    Near,
    Medium,
    Far,
}

fn apply_fog_preset(debug: &mut DebugSettings, preset: FogPreset) {
    match preset {
        FogPreset::Near => {
            debug.fog_start = 10.0;
            debug.fog_end = 32.0;
            debug.fog_visibility_distance = 28.0;
            debug.fog_density = 0.045;
        }
        FogPreset::Medium => {
            debug.fog_start = 22.0;
            debug.fog_end = 78.0;
            debug.fog_visibility_distance = 78.0;
            debug.fog_density = 0.0125;
        }
        FogPreset::Far => {
            debug.fog_start = 40.0;
            debug.fog_end = 160.0;
            debug.fog_visibility_distance = 150.0;
            debug.fog_density = 0.0045;
        }
    }
}

fn fog_linear_bounds(debug: &DebugSettings, anchor_offset: f32) -> (f32, f32) {
    let clear = debug.fog_clear_offset.max(0.0) + anchor_offset.max(0.0);
    let start = (debug.fog_start.max(0.0) + clear).max(0.0);
    let end = (debug.fog_end.max(debug.fog_start + 0.1) + clear).max(start + 0.1);
    (start, end)
}

fn density_from_visibility(visibility_distance: f32, transmittance: f32, squared: bool) -> f32 {
    let visibility = visibility_distance.max(0.1);
    let t = transmittance.clamp(0.001, 0.99);
    let neg_ln_t = -t.ln();
    let density = if squared {
        neg_ln_t.sqrt() / visibility
    } else {
        neg_ln_t / visibility
    };
    density.max(0.00001)
}

fn fog_density(debug: &DebugSettings, anchor_offset: f32, squared: bool) -> f32 {
    if debug.fog_use_visibility {
        let distance = (debug.fog_visibility_distance.max(0.1) + debug.fog_clear_offset.max(0.0))
            + anchor_offset.max(0.0);
        density_from_visibility(distance, debug.fog_visibility_transmittance, squared)
    } else {
        debug.fog_density.max(0.00001)
    }
}

fn fog_transmittance_for_distance(distance: f32, debug: &DebugSettings, anchor_offset: f32) -> f32 {
    let d = distance.max(0.0);
    match debug.fog_curve {
        FogCurveSetting::Linear => {
            let (start, end) = fog_linear_bounds(debug, anchor_offset);
            ((end - d) / (end - start).max(0.0001)).clamp(0.0, 1.0)
        }
        FogCurveSetting::Exponential => (-fog_density(debug, anchor_offset, false) * d)
            .exp()
            .clamp(0.0, 1.0),
        FogCurveSetting::ExponentialSquared => {
            let x = fog_density(debug, anchor_offset, true) * d;
            (-(x * x)).exp().clamp(0.0, 1.0)
        }
        FogCurveSetting::Atmospheric => (-fog_density(debug, anchor_offset, false) * d)
            .exp()
            .clamp(0.0, 1.0),
    }
}

fn distance_fog_from_debug(debug: &DebugSettings, anchor_offset: f32) -> DistanceFog {
    let (start, end) = fog_linear_bounds(debug, anchor_offset);
    let exp_density = fog_density(debug, anchor_offset, false);
    let exp2_density = fog_density(debug, anchor_offset, true);
    let falloff = match debug.fog_curve {
        FogCurveSetting::Linear => FogFalloff::Linear { start, end },
        FogCurveSetting::Exponential => FogFalloff::Exponential {
            density: exp_density,
        },
        FogCurveSetting::ExponentialSquared => FogFalloff::ExponentialSquared {
            density: exp2_density,
        },
        FogCurveSetting::Atmospheric => {
            let d = exp_density;
            FogFalloff::Atmospheric {
                extinction: Vec3::splat(d),
                inscattering: Vec3::splat(d),
            }
        }
    };

    let fog_color = (
        debug.fog_color.0.clamp(0.0, 1.0),
        debug.fog_color.1.clamp(0.0, 1.0),
        debug.fog_color.2.clamp(0.0, 1.0),
    );

    DistanceFog {
        // Keep values in linear space and let the render pipeline handle tonemapping afterwards.
        color: Color::linear_rgba(
            fog_color.0,
            fog_color.1,
            fog_color.2,
            debug.fog_opacity.clamp(0.0, 1.0),
        ),
        directional_light_color: Color::NONE,
        directional_light_exponent: 0.0,
        falloff,
    }
}

pub(super) fn fog_debug_sliders_ui(
    mut contexts: EguiContexts,
    menu: Res<MenuState>,
    settings: Res<GameSettings>,
    keybinds: Res<GameKeybinds>,
    mut debug: ResMut<DebugSettings>,
) {
    if !menu.open || menu.screen != MenuScreen::Debug {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let mut changed = false;

    egui::Window::new("Fog Settings")
        .collapsible(false)
        .resizable(false)
        .default_width(320.0)
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-18.0, 18.0))
        .show(ctx, |ui| {
            ui.label("Fog parameters (live)");

            let mut anchor = debug.fog_anchor;

            egui::ComboBox::from_label("Anchor")
                .selected_text(anchor.label())
                .show_ui(ui, |ui| {
                    changed |= ui
                        .selectable_value(
                            &mut anchor,
                            FogAnchorSetting::Character,
                            FogAnchorSetting::Character.label(),
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            &mut anchor,
                            FogAnchorSetting::Camera,
                            FogAnchorSetting::Camera.label(),
                        )
                        .changed();
                });
            if anchor != debug.fog_anchor {
                debug.fog_anchor = anchor;
                changed = true;
            }

            let mut curve = debug.fog_curve;
            egui::ComboBox::from_label("Curve")
                .selected_text(curve.label())
                .show_ui(ui, |ui| {
                    changed |= ui
                        .selectable_value(
                            &mut curve,
                            FogCurveSetting::Linear,
                            FogCurveSetting::Linear.label(),
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            &mut curve,
                            FogCurveSetting::Exponential,
                            FogCurveSetting::Exponential.label(),
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            &mut curve,
                            FogCurveSetting::ExponentialSquared,
                            FogCurveSetting::ExponentialSquared.label(),
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            &mut curve,
                            FogCurveSetting::Atmospheric,
                            FogCurveSetting::Atmospheric.label(),
                        )
                        .changed();
                });
            if curve != debug.fog_curve {
                debug.fog_curve = curve;
                changed = true;
            }

            let mut clear_offset = debug.fog_clear_offset;
            let clear_offset_changed = ui
                .add(egui::Slider::new(&mut clear_offset, 0.0..=80.0).text("Clear offset"))
                .on_hover_text("Extra heldere zone rond de anchor (camera of character).")
                .changed();
            if clear_offset_changed {
                debug.fog_clear_offset = clear_offset.max(0.0);
                changed = true;
            }

            let mut color = [debug.fog_color.0, debug.fog_color.1, debug.fog_color.2];
            ui.horizontal(|ui| {
                ui.label("Fog color");
                if ui.color_edit_button_rgb(&mut color).changed() {
                    debug.fog_color = (
                        color[0].clamp(0.0, 1.0),
                        color[1].clamp(0.0, 1.0),
                        color[2].clamp(0.0, 1.0),
                    );
                    changed = true;
                }
            });

            let mut opacity = debug.fog_opacity;
            let opacity_changed = ui
                .add(egui::Slider::new(&mut opacity, 0.0..=1.0).text("Fog alpha"))
                .on_hover_text("Maximale dekkingsgraad van mist.")
                .changed();
            if opacity_changed {
                debug.fog_opacity = opacity.clamp(0.0, 1.0);
                changed = true;
            }

            let mut hide_geometry = debug.fog_hide_geometry;
            if ui
                .checkbox(&mut hide_geometry, "Use alpha fog (no fog color)")
                .on_hover_text(
                    "Past fog-factor toe op materiaal-alpha (fade), in plaats van color-fog blend.",
                )
                .changed()
            {
                debug.fog_hide_geometry = hide_geometry;
                changed = true;
            }

            if debug.fog_curve == FogCurveSetting::Linear {
                let mut start = debug.fog_start;
                let start_changed = ui
                    .add(egui::Slider::new(&mut start, 0.0..=250.0).text("Start"))
                    .on_hover_text("Afstand waar lineaire mist begint.")
                    .changed();
                if start_changed {
                    debug.fog_start = start.max(0.0);
                    if debug.fog_end < debug.fog_start + 0.1 {
                        debug.fog_end = debug.fog_start + 0.1;
                    }
                    changed = true;
                }

                let mut end = debug.fog_end;
                let end_changed = ui
                    .add(egui::Slider::new(&mut end, (debug.fog_start + 0.1)..=400.0).text("End"))
                    .on_hover_text("Afstand waar lineaire mist volledig dekt.")
                    .changed();
                if end_changed {
                    debug.fog_end = end.max(debug.fog_start + 0.1);
                    changed = true;
                }
            } else {
                let mut use_visibility = debug.fog_use_visibility;
                if ui
                    .checkbox(&mut use_visibility, "Use visibility distance")
                    .on_hover_text(
                        "Rekent density uit met d = -ln(t)/V (of sqrt variant voor Exp2).",
                    )
                    .changed()
                {
                    debug.fog_use_visibility = use_visibility;
                    changed = true;
                }

                if debug.fog_use_visibility {
                    let mut visibility = debug.fog_visibility_distance;
                    if ui
                        .add(
                            egui::Slider::new(&mut visibility, 1.0..=500.0)
                                .text("Visibility distance"),
                        )
                        .on_hover_text("Gewenste zichtafstand V in world units.")
                        .changed()
                    {
                        debug.fog_visibility_distance = visibility.max(0.1);
                        changed = true;
                    }

                    let mut transmittance = debug.fog_visibility_transmittance;
                    if ui
                        .add(
                            egui::Slider::new(&mut transmittance, 0.001..=0.5)
                                .logarithmic(true)
                                .text("Transmittance @ V"),
                        )
                        .on_hover_text("Doel-transmittance t op afstand V.")
                        .changed()
                    {
                        debug.fog_visibility_transmittance = transmittance.clamp(0.001, 0.99);
                        changed = true;
                    }
                } else {
                    let mut density = debug.fog_density;
                    if ui
                        .add(
                            egui::Slider::new(&mut density, 0.00001..=0.2)
                                .logarithmic(true)
                                .text("Density"),
                        )
                        .on_hover_text("Handmatige density voor Exp/Exp2/Atmospheric.")
                        .changed()
                    {
                        debug.fog_density = density.max(0.00001);
                        changed = true;
                    }
                }
            }

            if debug.fog_anchor == FogAnchorSetting::Character {
                ui.small("Character-anchor compenseert camera-afstand.");
            }

            ui.separator();
            egui::CollapsingHeader::new("(i) Info: invloed per variabele")
                .default_open(false)
                .show(ui, |ui| {
                    ui.small("Anchor: bepaalt of fog meebeweegt met de camera of met de player.");
                    ui.small("Curve: linear, exp, exp2, of atmospheric falloff.");
                    ui.small("Linear gebruikt Start/End.");
                    ui.small("Exp/Exp2 gebruiken óf density óf visibility-model.");
                    ui.small("Clear offset voegt een heldere buffer rond de anchor toe.");
                    ui.small("Fog color + opacity sturen blendkleur en maximale dekking.");
                    ui.small(
                        "Use alpha fog schakelt color-fog uit en faded geometry via alpha/transmittance.",
                    );
                    ui.small(
                        "Distance metric is hier camera-range (euclidisch), niet view-space z.",
                    );
                });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Dichtbij").clicked() {
                    apply_fog_preset(&mut debug, FogPreset::Near);
                    changed = true;
                }
                if ui.button("Middel").clicked() {
                    apply_fog_preset(&mut debug, FogPreset::Medium);
                    changed = true;
                }
                if ui.button("Veraf").clicked() {
                    apply_fog_preset(&mut debug, FogPreset::Far);
                    changed = true;
                }
            });
        });

    if changed {
        save_persisted_config(&settings, &keybinds, &debug);
    }
}

fn create_debug_skybox_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let width = 1024usize;
    let height = 512usize;
    let mut data = vec![0_u8; width * height * 4];

    for y in 0..height {
        let v = y as f32 / (height - 1) as f32;
        for x in 0..width {
            let u = x as f32 / (width - 1) as f32;
            let idx = (y * width + x) * 4;

            let horizon = (v - 0.5).abs();
            let horizon_weight = (1.0 - (horizon * 4.0)).clamp(0.0, 1.0);

            let top = [0.18_f32, 0.30_f32, 0.52_f32];
            let bottom = [0.58_f32, 0.71_f32, 0.90_f32];
            let mut r = top[0] * v + bottom[0] * (1.0 - v);
            let mut g = top[1] * v + bottom[1] * (1.0 - v);
            let mut b = top[2] * v + bottom[2] * (1.0 - v);

            let checker = ((x / 48) + (y / 48)) % 2;
            let checker_boost = if checker == 0 { 0.06 } else { -0.03 };
            r = (r + checker_boost).clamp(0.0, 1.0);
            g = (g + checker_boost).clamp(0.0, 1.0);
            b = (b + checker_boost).clamp(0.0, 1.0);

            if x % 128 == 0 || y % 128 == 0 {
                r = 0.95;
                g = 0.25;
                b = 0.18;
            } else if x % 64 == 0 || y % 64 == 0 {
                r = (r + 0.25).clamp(0.0, 1.0);
                g = (g + 0.22).clamp(0.0, 1.0);
                b = (b + 0.18).clamp(0.0, 1.0);
            }

            if horizon < 0.01 {
                r = 1.0;
                g = 0.92;
                b = 0.35;
            } else if horizon_weight > 0.0 {
                r = (r + 0.12 * horizon_weight).clamp(0.0, 1.0);
                g = (g + 0.10 * horizon_weight).clamp(0.0, 1.0);
                b = (b + 0.06 * horizon_weight).clamp(0.0, 1.0);
            }

            if (u - 0.5).abs() < 0.0015 {
                r = 0.12;
                g = 0.98;
                b = 0.74;
            }

            data[idx] = (r * 255.0) as u8;
            data[idx + 1] = (g * 255.0) as u8;
            data[idx + 2] = (b * 255.0) as u8;
            data[idx + 3] = 255;
        }
    }

    let image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    );

    images.add(image)
}

pub(super) fn spawn_scenario_world(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
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
    let torso_mesh = meshes.add(Cuboid::new(0.54, 0.66, 0.30));
    let pelvis_mesh = meshes.add(Cuboid::new(0.42, 0.24, 0.26));
    let head_mesh = meshes.add(Cuboid::new(0.28, 0.30, 0.26));
    let hair_mesh = meshes.add(Cuboid::new(0.30, 0.10, 0.28));
    let upper_arm_len = 0.28_f32;
    let lower_arm_len = 0.26_f32;
    let upper_arm_mesh = meshes.add(Cuboid::new(0.12, upper_arm_len, 0.12));
    let lower_arm_mesh = meshes.add(Cuboid::new(0.11, lower_arm_len, 0.11));
    // Keep leg chain length aligned with hip height so IK can actually reach the floor.
    let upper_leg_len = 0.40_f32;
    let lower_leg_len = 0.40_f32;
    let ankle_height = 0.08_f32;
    let upper_leg_mesh = meshes.add(Cuboid::new(0.16, upper_leg_len, 0.16));
    let lower_leg_mesh = meshes.add(Cuboid::new(0.14, lower_leg_len, 0.14));
    let hand_mesh = meshes.add(Cuboid::new(0.11, 0.12, 0.10));
    let foot_mesh = meshes.add(Cuboid::new(0.14, 0.08, 0.24));

    let skin_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.93, 0.79, 0.67),
        perceptual_roughness: 0.9,
        ..default()
    });
    let shirt_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.38, 0.64),
        perceptual_roughness: 0.86,
        ..default()
    });
    let pants_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.17, 0.18, 0.23),
        perceptual_roughness: 0.9,
        ..default()
    });
    let hair_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.10, 0.07),
        perceptual_roughness: 0.82,
        ..default()
    });
    let boot_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.10, 0.10, 0.12),
        perceptual_roughness: 0.96,
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
    let skybox_texture = create_debug_skybox_texture(images);
    let skybox_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let skybox_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(skybox_texture),
        unlit: true,
        cull_mode: Some(bevy::render::render_resource::Face::Front),
        fog_enabled: false,
        ..default()
    });

    commands
        .spawn((
            Player::default(),
            Transform::from_xyz(0.0, player_half_height, 0.0),
            NotShadowCaster,
            PlayerCollider {
                radius: player_radius,
                half_height: player_half_height,
            },
            ProceduralHumanAnimState::from_position(Vec3::new(0.0, player_half_height, 0.0)),
            PlayerKinematics {
                horizontal_velocity: Vec2::ZERO,
                vertical_velocity: 0.0,
                grounded: true,
            },
            InGameEntity,
        ))
        .with_children(|player| {
            player
                .spawn((
                    ProceduralHumanVisualRoot,
                    Transform::from_xyz(0.0, -player_half_height, 0.0),
                ))
                .with_children(|human| {
                    human.spawn((
                        PlayerVisualPart,
                        Mesh3d(pelvis_mesh.clone()),
                        MeshMaterial3d(pants_mat.clone()),
                        Transform::from_xyz(0.0, 0.88, 0.0),
                    ));
                    human.spawn((
                        PlayerVisualPart,
                        Mesh3d(torso_mesh.clone()),
                        MeshMaterial3d(shirt_mat.clone()),
                        Transform::from_xyz(0.0, 1.24, 0.0),
                    ));
                    human
                        .spawn((
                            HumanHead {
                                base_local: Vec3::new(0.0, 1.64, 0.0),
                                max_yaw: 0.80,
                                max_pitch_up: 0.42,
                                max_pitch_down: 0.48,
                            },
                            PlayerVisualPart,
                            Mesh3d(head_mesh.clone()),
                            MeshMaterial3d(skin_mat.clone()),
                            Transform::from_xyz(0.0, 1.64, 0.0),
                        ))
                        .with_children(|head| {
                            head.spawn((
                                PlayerVisualPart,
                                Mesh3d(hair_mesh.clone()),
                                MeshMaterial3d(hair_mat.clone()),
                                Transform::from_xyz(0.0, 0.16, 0.0),
                            ));
                        });

                    let left_arm_base = Vec3::new(-0.34, 1.40, 0.0);
                    human
                        .spawn((
                            HumanArmPivot {
                                side: LimbSide::Left,
                                base_local: left_arm_base,
                                upper_len: upper_arm_len,
                                lower_len: lower_arm_len,
                            },
                            Transform::from_translation(left_arm_base),
                        ))
                        .with_children(|arm| {
                            arm.spawn((
                                PlayerVisualPart,
                                Mesh3d(upper_arm_mesh.clone()),
                                MeshMaterial3d(shirt_mat.clone()),
                                Transform::from_xyz(0.0, -upper_arm_len * 0.5, 0.0),
                            ));
                            arm.spawn((
                                HumanArmElbow,
                                Transform::from_xyz(0.0, -upper_arm_len, 0.0),
                            ))
                            .with_children(|elbow| {
                                elbow.spawn((
                                    PlayerVisualPart,
                                    Mesh3d(lower_arm_mesh.clone()),
                                    MeshMaterial3d(shirt_mat.clone()),
                                    Transform::from_xyz(0.0, -lower_arm_len * 0.5, 0.0),
                                ));
                                elbow.spawn((
                                    PlayerVisualPart,
                                    Mesh3d(hand_mesh.clone()),
                                    MeshMaterial3d(skin_mat.clone()),
                                    Transform::from_xyz(0.0, -(lower_arm_len + 0.07), 0.03),
                                ));
                            });
                        });

                    let right_arm_base = Vec3::new(0.34, 1.40, 0.0);
                    human
                        .spawn((
                            HumanArmPivot {
                                side: LimbSide::Right,
                                base_local: right_arm_base,
                                upper_len: upper_arm_len,
                                lower_len: lower_arm_len,
                            },
                            Transform::from_translation(right_arm_base),
                        ))
                        .with_children(|arm| {
                            arm.spawn((
                                PlayerVisualPart,
                                Mesh3d(upper_arm_mesh.clone()),
                                MeshMaterial3d(shirt_mat.clone()),
                                Transform::from_xyz(0.0, -upper_arm_len * 0.5, 0.0),
                            ));
                            arm.spawn((
                                HumanArmElbow,
                                Transform::from_xyz(0.0, -upper_arm_len, 0.0),
                            ))
                            .with_children(|elbow| {
                                elbow.spawn((
                                    PlayerVisualPart,
                                    Mesh3d(lower_arm_mesh.clone()),
                                    MeshMaterial3d(shirt_mat.clone()),
                                    Transform::from_xyz(0.0, -lower_arm_len * 0.5, 0.0),
                                ));
                                elbow.spawn((
                                    PlayerVisualPart,
                                    Mesh3d(hand_mesh.clone()),
                                    MeshMaterial3d(skin_mat.clone()),
                                    Transform::from_xyz(0.0, -(lower_arm_len + 0.07), 0.03),
                                ));
                            });
                        });

                    let left_leg_base = Vec3::new(-0.16, 0.88, 0.0);
                    human
                        .spawn((
                            HumanLegHip {
                                side: LimbSide::Left,
                                base_local: left_leg_base,
                                upper_len: upper_leg_len,
                                lower_len: lower_leg_len,
                                ankle_height,
                            },
                            Transform::from_translation(left_leg_base),
                        ))
                        .with_children(|leg| {
                            leg.spawn((
                                PlayerVisualPart,
                                Mesh3d(upper_leg_mesh.clone()),
                                MeshMaterial3d(pants_mat.clone()),
                                Transform::from_xyz(0.0, -upper_leg_len * 0.5, 0.0),
                            ));
                            leg.spawn((
                                HumanLegKnee,
                                Transform::from_xyz(0.0, -upper_leg_len, 0.0),
                            ))
                            .with_children(|knee| {
                                knee.spawn((
                                    PlayerVisualPart,
                                    Mesh3d(lower_leg_mesh.clone()),
                                    MeshMaterial3d(pants_mat.clone()),
                                    Transform::from_xyz(0.0, -lower_leg_len * 0.5, 0.0),
                                ));
                                knee.spawn((
                                    PlayerVisualPart,
                                    Mesh3d(foot_mesh.clone()),
                                    MeshMaterial3d(boot_mat.clone()),
                                    Transform::from_xyz(
                                        0.0,
                                        -(lower_leg_len + ankle_height * 0.5),
                                        0.09,
                                    ),
                                ));
                            });
                        });

                    let right_leg_base = Vec3::new(0.16, 0.88, 0.0);
                    human
                        .spawn((
                            HumanLegHip {
                                side: LimbSide::Right,
                                base_local: right_leg_base,
                                upper_len: upper_leg_len,
                                lower_len: lower_leg_len,
                                ankle_height,
                            },
                            Transform::from_translation(right_leg_base),
                        ))
                        .with_children(|leg| {
                            leg.spawn((
                                PlayerVisualPart,
                                Mesh3d(upper_leg_mesh.clone()),
                                MeshMaterial3d(pants_mat.clone()),
                                Transform::from_xyz(0.0, -upper_leg_len * 0.5, 0.0),
                            ));
                            leg.spawn((
                                HumanLegKnee,
                                Transform::from_xyz(0.0, -upper_leg_len, 0.0),
                            ))
                            .with_children(|knee| {
                                knee.spawn((
                                    PlayerVisualPart,
                                    Mesh3d(lower_leg_mesh.clone()),
                                    MeshMaterial3d(pants_mat.clone()),
                                    Transform::from_xyz(0.0, -lower_leg_len * 0.5, 0.0),
                                ));
                                knee.spawn((
                                    PlayerVisualPart,
                                    Mesh3d(foot_mesh.clone()),
                                    MeshMaterial3d(boot_mat.clone()),
                                    Transform::from_xyz(
                                        0.0,
                                        -(lower_leg_len + ankle_height * 0.5),
                                        0.09,
                                    ),
                                ));
                            });
                        });
                });
        });

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
        PrimaryEguiContext,
        Transform::from_xyz(0.0, 4.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ThirdPersonCameraRig::default(),
        Msaa::Sample4,
        default_distance_fog(),
        InGameEntity,
    ));

    commands.spawn((
        SkyboxCube,
        Mesh3d(skybox_mesh),
        MeshMaterial3d(skybox_mat),
        Transform::from_scale(Vec3::splat((ground_extent * 18.0).max(2000.0))),
        NotShadowCaster,
        NotShadowReceiver,
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
        GroundPlane,
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
    in_game_entities: Query<Entity, With<InGameEntity>>,
    start_menu_roots: Query<Entity, With<StartMenuRoot>>,
    start_menu_cameras: Query<Entity, With<StartMenuCamera>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if !menu.open {
        return;
    }

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
                    }
                    MenuButtonAction::ToggleMsaa => {
                        settings.msaa_enabled = !settings.msaa_enabled;
                    }
                    MenuButtonAction::ToggleShadowMode => {
                        settings.shadow_mode = settings.shadow_mode.next();
                    }
                    MenuButtonAction::TogglePerformanceOverlay => {
                        debug.show_performance_overlay = !debug.show_performance_overlay;
                    }
                    MenuButtonAction::ToggleBakedShadows => {
                        debug.show_baked_shadows = !debug.show_baked_shadows;
                    }
                    MenuButtonAction::ToggleFog => {
                        debug.show_fog = !debug.show_fog;
                    }
                    MenuButtonAction::ToggleCollisionShapes => {
                        debug.show_collision_shapes = !debug.show_collision_shapes;
                    }
                    MenuButtonAction::ToggleAnimationDebug => {
                        debug.show_animation_debug = !debug.show_animation_debug;
                    }
                    MenuButtonAction::ToggleWireframe => {
                        debug.show_wireframe = !debug.show_wireframe;
                    }
                    MenuButtonAction::ToggleWorldAxes => {
                        debug.show_world_axes = !debug.show_world_axes;
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
}

pub(super) fn capture_rebind_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<MenuState>,
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

        if keybinds.has_key(action, *key) {
            keybinds.remove_key(action, *key)
        } else {
            keybinds.add_key(action, *key)
        };
        menu.awaiting_rebind = None;
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

pub(super) fn persist_config_on_change(
    settings: Res<GameSettings>,
    keybinds: Res<GameKeybinds>,
    debug: Res<DebugSettings>,
) {
    if settings.is_changed() || keybinds.is_changed() || debug.is_changed() {
        save_persisted_config(&settings, &keybinds, &debug);
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
                left: px(16),
                bottom: px(16),
                ..default()
            },
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
                                MenuButton(MenuButtonAction::ToggleAnimationDebug),
                                menu_button_node(),
                                menu_button_normal_color(),
                            ))
                            .with_child(Text::new(format!(
                                "Animation Rig: {}",
                                if debug.show_animation_debug {
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
    camera_entities: Query<(Entity, &Transform), With<Camera3d>>,
    player_transforms: Query<&Transform, With<Player>>,
    player_entities: Query<(Entity, Has<NotShadowCaster>), With<Player>>,
    player_visual_entities: Query<(Entity, Has<NotShadowCaster>), With<PlayerVisualPart>>,
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
                Without<SkyboxCube>,
            ),
        >,
        Query<
            Entity,
            (
                With<Mesh3d>,
                With<Wireframe>,
                Without<PlayerBlobShadow>,
                Without<BakedShadow>,
                Without<SkyboxCube>,
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

    if let Ok((camera, camera_transform)) = camera_entities.single() {
        if settings.msaa_enabled {
            commands.entity(camera).insert(Msaa::Sample4);
        } else {
            commands.entity(camera).insert(Msaa::Off);
        }

        let anchor_offset = if debug.fog_anchor == FogAnchorSetting::Character {
            player_transforms
                .single()
                .map(|player_transform| {
                    camera_transform
                        .translation
                        .distance(player_transform.translation)
                })
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let has_fog = camera_has_fog.get(camera).is_ok();
        if debug.show_fog && !debug.fog_hide_geometry {
            commands
                .entity(camera)
                .insert(distance_fog_from_debug(&debug, anchor_offset));
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

    for (entity, has_not_shadow_caster) in &player_visual_entities {
        if stencil_mode {
            if has_not_shadow_caster {
                commands.entity(entity).remove::<NotShadowCaster>();
            }
        } else if !has_not_shadow_caster {
            commands.entity(entity).insert(NotShadowCaster);
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

pub(super) fn apply_fog_alpha_materials(
    time: Res<Time>,
    debug: Res<DebugSettings>,
    camera_query: Query<&Transform, With<Camera3d>>,
    player_transforms: Query<&Transform, With<Player>>,
    mut mesh_materials: Query<
        (
            Entity,
            &GlobalTransform,
            Has<GroundPlane>,
            &mut MeshMaterial3d<StandardMaterial>,
            Option<&mut FogAlphaMaterialState>,
        ),
        (
            With<Mesh3d>,
            Without<SkyboxCube>,
            Without<PlayerBlobShadow>,
            Without<BakedShadow>,
        ),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let alpha_mode = debug.show_fog && debug.fog_hide_geometry;
    let anchor_offset = if debug.fog_anchor == FogAnchorSetting::Character {
        player_transforms
            .single()
            .map(|player_transform| {
                camera_transform
                    .translation
                    .distance(player_transform.translation)
            })
            .unwrap_or(0.0)
    } else {
        0.0
    };

    let smooth = 1.0 - (-time.delta_secs() * 10.0).exp();

    for (entity, transform, is_ground, mut material_handle, state) in &mut mesh_materials {
        if !alpha_mode {
            let Some(mut state) = state else {
                continue;
            };
            let Some(material) = materials.get_mut(&material_handle.0) else {
                continue;
            };
            let linear = material.base_color.to_linear();
            material.base_color =
                Color::linear_rgba(linear.red, linear.green, linear.blue, state.base_alpha);
            material.alpha_mode = state.original_alpha_mode.clone();
            material.fog_enabled = state.original_fog_enabled;
            state.current_alpha_factor = 1.0;
            continue;
        }

        if state.is_none() {
            let Some(source_material) = materials.get(&material_handle.0).cloned() else {
                continue;
            };
            let linear = source_material.base_color.to_linear();
            let state = FogAlphaMaterialState {
                base_alpha: linear.alpha,
                current_alpha_factor: 1.0,
                original_alpha_mode: source_material.alpha_mode.clone(),
                original_fog_enabled: source_material.fog_enabled,
            };
            material_handle.0 = materials.add(source_material);
            commands.entity(entity).insert(state);
            continue;
        }

        let Some(mut state) = state else {
            continue;
        };
        let Some(material) = materials.get_mut(&material_handle.0) else {
            continue;
        };

        let distance = transform
            .translation()
            .distance(camera_transform.translation);
        let transmittance = fog_transmittance_for_distance(distance, &debug, anchor_offset);
        let fog_intensity = (1.0 - transmittance).clamp(0.0, 1.0);
        let target_alpha_factor = 1.0 - fog_intensity * debug.fog_opacity.clamp(0.0, 1.0);
        state.current_alpha_factor += (target_alpha_factor - state.current_alpha_factor) * smooth;
        let target_alpha = (state.base_alpha * state.current_alpha_factor).clamp(0.0, 1.0);

        let linear = material.base_color.to_linear();
        material.base_color =
            Color::linear_rgba(linear.red, linear.green, linear.blue, target_alpha);
        material.alpha_mode = if is_ground {
            AlphaMode::AlphaToCoverage
        } else {
            AlphaMode::Blend
        };
        material.fog_enabled = false;
    }
}
