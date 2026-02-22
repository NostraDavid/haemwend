use bevy::app::AppExit;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::window::{
    CursorGrabMode, CursorOptions, MonitorSelection, PresentMode, PrimaryWindow, VideoModeSelection,
    WindowMode, WindowResolution,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const CONFIG_PATH: &str = "config/game_config.ron";
const SCENARIOS_PATH_DEFAULT: &str = "config/scenarios";
const RESOLUTION_OPTIONS: &[(u32, u32)] = &[
    (1280, 720),
    (1600, 900),
    (1920, 1080),
    (2560, 1440),
    (3440, 1440),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScenarioDefinition {
    id: String,
    name: String,
    description: String,
    ground_extent: f32,
    crate_grid_radius: i32,
    crate_spacing: f32,
    crate_pattern_mod: i32,
    wall_count: i32,
    wall_spacing: f32,
    wall_z: f32,
    tower_z: f32,
    sun_position: [f32; 3],
}

impl ScenarioDefinition {
    fn sun_vec3(&self) -> Vec3 {
        Vec3::new(
            self.sun_position[0],
            self.sun_position[1],
            self.sun_position[2],
        )
    }
}

#[derive(Resource, Debug, Clone)]
struct ScenarioCatalog {
    scenarios: Vec<ScenarioDefinition>,
}

impl ScenarioCatalog {
    fn index_by_id(&self, id: &str) -> Option<usize> {
        self.scenarios.iter().position(|scenario| scenario.id == id)
    }
}

#[derive(Debug, Clone)]
struct CliOptions {
    scenario_id: Option<String>,
    scenarios_path: String,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            scenario_id: None,
            scenarios_path: SCENARIOS_PATH_DEFAULT.to_string(),
        }
    }
}

#[derive(Resource, Debug)]
struct GameFlowState {
    in_game: bool,
    pending_scenario: Option<usize>,
}

impl Default for GameFlowState {
    fn default() -> Self {
        Self {
            in_game: false,
            pending_scenario: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum DisplayModeSetting {
    Windowed,
    FullscreenWindowed,
    FullscreenExclusive,
}

impl DisplayModeSetting {
    fn next(self) -> Self {
        match self {
            Self::Windowed => Self::FullscreenWindowed,
            Self::FullscreenWindowed => Self::FullscreenExclusive,
            Self::FullscreenExclusive => Self::Windowed,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Windowed => "Windowed",
            Self::FullscreenWindowed => "Fullscreen Windowed",
            Self::FullscreenExclusive => "Fullscreen",
        }
    }

    fn to_window_mode(self) -> WindowMode {
        match self {
            Self::Windowed => WindowMode::Windowed,
            Self::FullscreenWindowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            Self::FullscreenExclusive => {
                WindowMode::Fullscreen(MonitorSelection::Current, VideoModeSelection::Current)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum ShadowModeSetting {
    Blob,
    Stencil,
}

impl ShadowModeSetting {
    fn next(self) -> Self {
        match self {
            Self::Blob => Self::Stencil,
            Self::Stencil => Self::Blob,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Blob => "Blob",
            Self::Stencil => "Stencil",
        }
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
struct GameSettings {
    display_mode: DisplayModeSetting,
    resolution_width: u32,
    resolution_height: u32,
    msaa_enabled: bool,
    shadow_mode: ShadowModeSetting,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            display_mode: DisplayModeSetting::Windowed,
            resolution_width: 1920,
            resolution_height: 1080,
            msaa_enabled: true,
            shadow_mode: ShadowModeSetting::Blob,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameAction {
    MoveForward,
    MoveBackward,
    StrafeLeft,
    StrafeRight,
    TurnLeft,
    TurnRight,
    Sprint,
    Jump,
}

const ACTION_ORDER: [GameAction; 8] = [
    GameAction::MoveForward,
    GameAction::MoveBackward,
    GameAction::StrafeLeft,
    GameAction::StrafeRight,
    GameAction::TurnLeft,
    GameAction::TurnRight,
    GameAction::Sprint,
    GameAction::Jump,
];

impl GameAction {
    fn label(self) -> &'static str {
        match self {
            Self::MoveForward => "Move Forward",
            Self::MoveBackward => "Move Backward",
            Self::StrafeLeft => "Strafe Left",
            Self::StrafeRight => "Strafe Right",
            Self::TurnLeft => "Turn Left",
            Self::TurnRight => "Turn Right",
            Self::Sprint => "Sprint",
            Self::Jump => "Jump",
        }
    }
}

#[derive(Resource, Debug, Clone)]
struct GameKeybinds {
    move_forward: Vec<KeyCode>,
    move_backward: Vec<KeyCode>,
    strafe_left: Vec<KeyCode>,
    strafe_right: Vec<KeyCode>,
    turn_left: Vec<KeyCode>,
    turn_right: Vec<KeyCode>,
    sprint: Vec<KeyCode>,
    jump: Vec<KeyCode>,
}

impl Default for GameKeybinds {
    fn default() -> Self {
        Self {
            move_forward: vec![KeyCode::KeyW],
            move_backward: vec![KeyCode::KeyS],
            strafe_left: vec![KeyCode::KeyQ],
            strafe_right: vec![KeyCode::KeyE],
            turn_left: vec![KeyCode::KeyA],
            turn_right: vec![KeyCode::KeyD],
            sprint: vec![KeyCode::ShiftLeft],
            jump: vec![KeyCode::Space],
        }
    }
}

impl GameKeybinds {
    fn keys_for(&self, action: GameAction) -> &[KeyCode] {
        match action {
            GameAction::MoveForward => &self.move_forward,
            GameAction::MoveBackward => &self.move_backward,
            GameAction::StrafeLeft => &self.strafe_left,
            GameAction::StrafeRight => &self.strafe_right,
            GameAction::TurnLeft => &self.turn_left,
            GameAction::TurnRight => &self.turn_right,
            GameAction::Sprint => &self.sprint,
            GameAction::Jump => &self.jump,
        }
    }

    fn keys_for_mut(&mut self, action: GameAction) -> &mut Vec<KeyCode> {
        match action {
            GameAction::MoveForward => &mut self.move_forward,
            GameAction::MoveBackward => &mut self.move_backward,
            GameAction::StrafeLeft => &mut self.strafe_left,
            GameAction::StrafeRight => &mut self.strafe_right,
            GameAction::TurnLeft => &mut self.turn_left,
            GameAction::TurnRight => &mut self.turn_right,
            GameAction::Sprint => &mut self.sprint,
            GameAction::Jump => &mut self.jump,
        }
    }

    fn action_pressed(&self, input: &ButtonInput<KeyCode>, action: GameAction) -> bool {
        self.keys_for(action).iter().any(|key| input.pressed(*key))
    }

    fn action_just_pressed(&self, input: &ButtonInput<KeyCode>, action: GameAction) -> bool {
        self.keys_for(action)
            .iter()
            .any(|key| input.just_pressed(*key))
    }

    fn add_key(&mut self, action: GameAction, key: KeyCode) -> bool {
        let keys = self.keys_for_mut(action);
        if keys.contains(&key) {
            return false;
        }

        keys.push(key);
        true
    }

    fn remove_key(&mut self, action: GameAction, key: KeyCode) -> bool {
        let keys = self.keys_for_mut(action);
        if keys.len() <= 1 {
            return false;
        }

        let old_len = keys.len();
        keys.retain(|k| *k != key);
        old_len != keys.len()
    }

    fn has_key(&self, action: GameAction, key: KeyCode) -> bool {
        self.keys_for(action).contains(&key)
    }

    fn display_keys(&self, action: GameAction) -> String {
        self.keys_for(action)
            .iter()
            .map(|key| keycode_to_label(*key))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn ensure_non_empty(&mut self) {
        for action in ACTION_ORDER {
            if self.keys_for(action).is_empty() {
                let fallback = GameKeybinds::default();
                self.keys_for_mut(action)
                    .push(fallback.keys_for(action)[0]);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedKeybinds {
    move_forward: String,
    move_backward: String,
    strafe_left: String,
    strafe_right: String,
    turn_left: String,
    turn_right: String,
    sprint: String,
    jump: String,
}

impl Default for PersistedKeybinds {
    fn default() -> Self {
        Self::from_runtime(&GameKeybinds::default())
    }
}

impl PersistedKeybinds {
    fn from_runtime(bindings: &GameKeybinds) -> Self {
        Self {
            move_forward: keycodes_to_names(bindings.keys_for(GameAction::MoveForward)),
            move_backward: keycodes_to_names(bindings.keys_for(GameAction::MoveBackward)),
            strafe_left: keycodes_to_names(bindings.keys_for(GameAction::StrafeLeft)),
            strafe_right: keycodes_to_names(bindings.keys_for(GameAction::StrafeRight)),
            turn_left: keycodes_to_names(bindings.keys_for(GameAction::TurnLeft)),
            turn_right: keycodes_to_names(bindings.keys_for(GameAction::TurnRight)),
            sprint: keycodes_to_names(bindings.keys_for(GameAction::Sprint)),
            jump: keycodes_to_names(bindings.keys_for(GameAction::Jump)),
        }
    }

    fn to_runtime(&self) -> GameKeybinds {
        let mut runtime = GameKeybinds {
            move_forward: keycodes_from_names(&self.move_forward),
            move_backward: keycodes_from_names(&self.move_backward),
            strafe_left: keycodes_from_names(&self.strafe_left),
            strafe_right: keycodes_from_names(&self.strafe_right),
            turn_left: keycodes_from_names(&self.turn_left),
            turn_right: keycodes_from_names(&self.turn_right),
            sprint: keycodes_from_names(&self.sprint),
            jump: keycodes_from_names(&self.jump),
        };
        runtime.ensure_non_empty();
        runtime
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedConfig {
    settings: GameSettings,
    keybinds: PersistedKeybinds,
}

impl Default for PersistedConfig {
    fn default() -> Self {
        Self {
            settings: GameSettings::default(),
            keybinds: PersistedKeybinds::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuScreen {
    Main,
    Settings,
    Keybinds,
    ExitConfirm,
}

#[derive(Resource, Debug)]
struct MenuState {
    open: bool,
    screen: MenuScreen,
    awaiting_rebind: Option<GameAction>,
    keybind_filter: String,
    dirty: bool,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            open: false,
            screen: MenuScreen::Main,
            awaiting_rebind: None,
            keybind_filter: String::new(),
            dirty: false,
        }
    }
}

#[derive(Resource, Debug, Default)]
struct MouseLookCaptureState {
    active: bool,
    restore_position: Option<Vec2>,
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

#[derive(Component)]
struct PlayerBlobShadow;

#[derive(Component)]
struct BakedShadow;

#[derive(Component)]
struct MenuRoot;

#[derive(Component)]
struct InGameEntity;

#[derive(Component)]
struct StartMenuRoot;

#[derive(Component)]
struct StartMenuCamera;

#[derive(Component, Clone, Copy)]
struct MenuButton(MenuButtonAction);

#[derive(Component, Clone, Copy)]
struct StartMenuButton(StartMenuButtonAction);

#[derive(Clone, Copy)]
enum StartMenuButtonAction {
    StartScenario(usize),
    ExitGame,
}

#[derive(Clone, Copy)]
enum MenuButtonAction {
    Resume,
    OpenSettings,
    OpenKeybinds,
    OpenExitConfirm,
    BackMain,
    ExitNow,
    CycleDisplayMode,
    CycleResolution,
    ToggleMsaa,
    ToggleShadowMode,
    StartRebind(GameAction),
    ClearKeybindFilter,
}

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

fn main() {
    let cli = parse_cli_options();
    let scenario_catalog = load_scenario_catalog(Path::new(&cli.scenarios_path));
    let pending_scenario = if let Some(requested_id) = cli.scenario_id.as_deref() {
        match scenario_catalog.index_by_id(requested_id) {
            Some(index) => Some(index),
            None => {
                let available = scenario_catalog
                    .scenarios
                    .iter()
                    .map(|scenario| scenario.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                eprintln!(
                    "Scenario '{}' niet gevonden. Beschikbaar: {}",
                    requested_id, available
                );
                None
            }
        }
    } else {
        None
    };

    let persisted = load_persisted_config();
    let initial_settings = persisted.settings;
    let initial_keybinds = persisted.keybinds.to_runtime();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "haemwend".into(),
                resolution: WindowResolution::new(1920, 1080),
                present_mode: PresentMode::Immediate,
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
        .insert_resource(initial_settings)
        .insert_resource(initial_keybinds)
        .insert_resource(GameFlowState {
            in_game: false,
            pending_scenario,
        })
        .insert_resource(scenario_catalog)
        .insert_resource(MenuState::default())
        .insert_resource(MouseLookCaptureState::default())
        .insert_resource(ClearColor(Color::srgb(0.57, 0.70, 0.92)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.56, 0.61, 0.67),
            brightness: 135.0,
            affects_lightmapped_meshes: true,
        })
        .add_systems(Startup, setup_start_menu)
        .add_systems(Update, (handle_start_menu_buttons, load_pending_scenario).chain())
        .add_systems(
            Update,
            (
                toggle_menu_on_escape,
                handle_menu_buttons,
                capture_rebind_input,
                capture_keybind_filter_input,
                apply_runtime_settings,
                rebuild_menu_ui,
            )
                .chain(),
        )
        .add_systems(Update, sync_mouse_capture_with_focus)
        .add_systems(
            Update,
            (player_move, update_player_blob_shadow, third_person_camera)
                .chain()
                .after(rebuild_menu_ui),
        )
        .add_systems(Update, update_performance_overlay)
        .run();
}

fn setup_start_menu(
    mut commands: Commands,
    flow: Res<GameFlowState>,
    scenarios: Res<ScenarioCatalog>,
) {
    if flow.pending_scenario.is_none() {
        commands.spawn((Camera2d, StartMenuCamera));
        spawn_start_menu_ui(&mut commands, &scenarios);
    }
}

fn spawn_start_menu_ui(commands: &mut Commands, scenarios: &ScenarioCatalog) {
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

fn handle_start_menu_buttons(
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

fn load_pending_scenario(
    mut commands: Commands,
    mut flow: ResMut<GameFlowState>,
    scenarios: Res<ScenarioCatalog>,
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

    spawn_scenario_world(&mut commands, &mut meshes, &mut materials, &scenario);
    settings.set_changed();
    flow.in_game = true;
}

fn spawn_scenario_world(
    commands: &mut Commands,
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

    let player_mesh = meshes.add(Cuboid::new(0.8, 1.8, 0.8));
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
        Transform::from_xyz(0.0, 0.9, 0.0),
        NotShadowCaster,
        PlayerCollider {
            half_extents: Vec3::new(0.35, 0.9, 0.35),
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
        DistanceFog {
            color: Color::srgba(0.62, 0.72, 0.84, 1.0),
            directional_light_color: Color::srgba(0.97, 0.88, 0.70, 0.5),
            directional_light_exponent: 20.0,
            falloff: FogFalloff::Linear {
                start: 22.0,
                end: 78.0,
            },
        },
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

    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(ground_mat),
        Transform::from_xyz(0.0, -0.05, 0.0),
        WorldCollider {
            half_extents: Vec3::new(ground_extent * 0.5, 0.05, ground_extent * 0.5),
        },
        InGameEntity,
    ));

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
                spawn_baked_shadow(
                    commands,
                    &baked_shadow_mesh,
                    &baked_shadow_mat,
                    Vec3::new(
                        x as f32 * crate_spacing,
                        0.011,
                        z as f32 * crate_spacing,
                    ),
                    Vec2::new(1.25, 1.25),
                );
            }
        }
    }

    for i in -wall_count..=wall_count {
        commands.spawn((
            Mesh3d(wall_mesh.clone()),
            MeshMaterial3d(wall_mat.clone()),
            Transform::from_xyz(i as f32 * wall_spacing, 1.5, wall_z),
            NotShadowCaster,
            WorldCollider {
                half_extents: Vec3::splat(1.5),
            },
            InGameEntity,
        ));
        spawn_baked_shadow(
            commands,
            &baked_shadow_mesh,
            &baked_shadow_mat,
            Vec3::new(i as f32 * wall_spacing, 0.011, wall_z),
            Vec2::new(3.4, 3.0),
        );
    }

    commands.spawn((
        Mesh3d(tower_mesh),
        MeshMaterial3d(tower_mat),
        Transform::from_xyz(0.0, 4.0, tower_z),
        NotShadowCaster,
        WorldCollider {
            half_extents: Vec3::new(2.0, 4.0, 2.0),
        },
        InGameEntity,
    ));
    spawn_baked_shadow(
        commands,
        &baked_shadow_mesh,
        &baked_shadow_mat,
        Vec3::new(0.0, 0.011, tower_z),
        Vec2::new(5.0, 5.0),
    );

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

fn toggle_menu_on_escape(
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

fn handle_menu_buttons(
    mut interactions: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut commands: Commands,
    scenarios: Res<ScenarioCatalog>,
    mut flow: ResMut<GameFlowState>,
    mut menu: ResMut<MenuState>,
    mut settings: ResMut<GameSettings>,
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
        save_persisted_config(&settings, &keybinds);
    }
}

fn capture_rebind_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<MenuState>,
    settings: Res<GameSettings>,
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
            save_persisted_config(&settings, &keybinds);
        }
        menu.dirty = true;
        break;
    }
}

fn capture_keybind_filter_input(
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

fn rebuild_menu_ui(
    mut commands: Commands,
    flow: Res<GameFlowState>,
    mut menu: ResMut<MenuState>,
    existing_roots: Query<Entity, With<MenuRoot>>,
    settings: Res<GameSettings>,
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
                                "Shadows: {}",
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

fn apply_runtime_settings(
    settings: Res<GameSettings>,
    primary_window: Single<&mut Window, With<PrimaryWindow>>,
    camera_entities: Query<Entity, With<Camera3d>>,
    player_entities: Query<(Entity, Has<NotShadowCaster>), With<Player>>,
    mut lights: Query<&mut DirectionalLight>,
    mut blob_visibility: Query<&mut Visibility, (With<PlayerBlobShadow>, Without<BakedShadow>)>,
    mut baked_visibility: Query<&mut Visibility, (With<BakedShadow>, Without<PlayerBlobShadow>)>,
    mut commands: Commands,
) {
    if !settings.is_changed() {
        return;
    }

    let mut window = primary_window.into_inner();
    window.mode = settings.display_mode.to_window_mode();
    window
        .resolution
        .set(settings.resolution_width as f32, settings.resolution_height as f32);

    if let Ok(camera) = camera_entities.single() {
        if settings.msaa_enabled {
            commands.entity(camera).insert(Msaa::Sample4);
        } else {
            commands.entity(camera).insert(Msaa::Off);
        }
    }

    let stencil_mode = settings.shadow_mode == ShadowModeSetting::Stencil;

    for mut light in &mut lights {
        light.shadows_enabled = stencil_mode;
    }

    for mut visibility in &mut blob_visibility {
        *visibility = if stencil_mode {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }

    for mut visibility in &mut baked_visibility {
        *visibility = if stencil_mode {
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
}

fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    menu: Res<MenuState>,
    keybinds: Res<GameKeybinds>,
    camera_query: Query<&ThirdPersonCameraRig, With<Camera3d>>,
    mut player_query: Query<(&mut Transform, &Player, &PlayerCollider, &mut PlayerKinematics)>,
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
            - keybinds.action_pressed(&keys, GameAction::TurnLeft) as i8) as f32;
        if turn_axis != 0.0 {
            transform.rotate_y(-turn_axis * player.turn_speed * dt);
        }
    }

    let forward = transform.rotation * -Vec3::Z;
    let right = transform.rotation * Vec3::X;

    let forward_axis = (keybinds.action_pressed(&keys, GameAction::MoveForward) as i8
        - keybinds.action_pressed(&keys, GameAction::MoveBackward) as i8) as f32;

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

fn third_person_camera(
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
    rig.distance =
        (rig.distance - mouse_scroll.delta.y * rig.zoom_sensitivity).clamp(rig.min_distance, rig.max_distance);

    let target = player_transform.translation;
    let rotation = Quat::from_euler(EulerRot::YXZ, rig.yaw, rig.pitch, 0.0);
    let orbit_offset = rotation * Vec3::new(0.0, 0.0, rig.distance);

    camera_transform.translation = target + orbit_offset + Vec3::Y * rig.height;
    camera_transform.look_at(target + Vec3::Y * rig.focus_height, Vec3::Y);
}

fn update_player_blob_shadow(
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
            let inside_x = (player_pos.x - world_pos.x).abs() <= world_collider.half_extents.x + 0.45;
            let inside_z = (player_pos.z - world_pos.z).abs() <= world_collider.half_extents.z + 0.45;
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

fn sync_mouse_capture_with_focus(
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

fn menu_button_node() -> Node {
    Node {
        width: percent(100),
        height: px(40),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        margin: UiRect::bottom(px(6)),
        ..default()
    }
}

fn menu_button_normal_color() -> BackgroundColor {
    BackgroundColor(Color::srgb(0.17, 0.20, 0.26))
}

fn menu_button_hover_color() -> BackgroundColor {
    BackgroundColor(Color::srgb(0.23, 0.27, 0.35))
}

fn menu_button_pressed_color() -> BackgroundColor {
    BackgroundColor(Color::srgb(0.19, 0.43, 0.25))
}

fn spawn_baked_shadow(
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

fn parse_cli_options() -> CliOptions {
    let mut options = CliOptions::default();
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scenario" | "-s" => {
                let Some(value) = args.next() else {
                    eprintln!("--scenario verwacht een scenario-id");
                    print_cli_help_and_exit(2);
                };
                options.scenario_id = Some(value);
            }
            "--scenarios-dir" | "--scenarios-path" | "--scenarios-file" => {
                let Some(value) = args.next() else {
                    eprintln!("{arg} verwacht een pad");
                    print_cli_help_and_exit(2);
                };
                options.scenarios_path = value;
            }
            "--help" | "-h" => {
                print_cli_help_and_exit(0);
            }
            _ => {
                eprintln!("Onbekende optie: {arg}");
                print_cli_help_and_exit(2);
            }
        }
    }

    options
}

fn print_cli_help_and_exit(code: i32) -> ! {
    println!(
        "Gebruik:\n  haemwend [opties]\n\nOpties:\n  -s, --scenario <id>         Start direct met scenario-id\n      --scenarios-dir <pad>   Map met scenario-bestanden (1 .ron per scenario)\n      --scenarios-path <pad>  Alias voor --scenarios-dir\n      --scenarios-file <pad>  Legacy alias (ondersteunt ook 1 bestand)\n  -h, --help                  Toon hulp"
    );
    std::process::exit(code);
}

fn default_scenarios() -> Vec<ScenarioDefinition> {
    vec![
        ScenarioDefinition {
            id: "greenwood".to_string(),
            name: "Greenwood Valley".to_string(),
            description: "Open veld met verspreide kratten en muursegmenten.".to_string(),
            ground_extent: 120.0,
            crate_grid_radius: 8,
            crate_spacing: 3.0,
            crate_pattern_mod: 4,
            wall_count: 5,
            wall_spacing: 3.2,
            wall_z: -20.0,
            tower_z: -30.0,
            sun_position: [18.0, 24.0, 12.0],
        },
        ScenarioDefinition {
            id: "arena".to_string(),
            name: "Iron Arena".to_string(),
            description: "Compacte arena met dichter op elkaar staande obstakels.".to_string(),
            ground_extent: 80.0,
            crate_grid_radius: 6,
            crate_spacing: 2.6,
            crate_pattern_mod: 3,
            wall_count: 7,
            wall_spacing: 2.6,
            wall_z: -16.0,
            tower_z: -24.0,
            sun_position: [14.0, 20.0, 10.0],
        },
        ScenarioDefinition {
            id: "canyon".to_string(),
            name: "Red Canyon".to_string(),
            description: "Langgerekte map met pilaren en sterke dieptewerking.".to_string(),
            ground_extent: 180.0,
            crate_grid_radius: 10,
            crate_spacing: 3.4,
            crate_pattern_mod: 5,
            wall_count: 9,
            wall_spacing: 3.5,
            wall_z: -30.0,
            tower_z: -42.0,
            sun_position: [22.0, 30.0, 14.0],
        },
        ScenarioDefinition {
            id: "gauntlet".to_string(),
            name: "Stone Gauntlet".to_string(),
            description: "Smalle route met dichte obstakels voor korte, intensieve runs.".to_string(),
            ground_extent: 72.0,
            crate_grid_radius: 5,
            crate_spacing: 2.2,
            crate_pattern_mod: 2,
            wall_count: 11,
            wall_spacing: 2.1,
            wall_z: -14.0,
            tower_z: -20.0,
            sun_position: [12.0, 18.0, 8.0],
        },
        ScenarioDefinition {
            id: "highlands".to_string(),
            name: "Frost Highlands".to_string(),
            description: "Grote open vlakte met weinig dekking en lange zichtlijnen.".to_string(),
            ground_extent: 240.0,
            crate_grid_radius: 12,
            crate_spacing: 4.2,
            crate_pattern_mod: 6,
            wall_count: 4,
            wall_spacing: 5.5,
            wall_z: -40.0,
            tower_z: -58.0,
            sun_position: [28.0, 35.0, 16.0],
        },
    ]
}

fn is_ron_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ron"))
}

fn filter_valid_scenarios(
    mut scenarios: Vec<ScenarioDefinition>,
    source: &str,
) -> Vec<ScenarioDefinition> {
    scenarios.retain(|scenario| !scenario.id.trim().is_empty() && !scenario.name.trim().is_empty());
    if scenarios.is_empty() {
        eprintln!("Scenario-bron ({source}) bevat geen geldige scenario's");
    }
    scenarios
}

fn write_default_scenarios_to_dir(path: &Path) -> bool {
    if let Err(err) = fs::create_dir_all(path) {
        eprintln!("Kon scenario-map niet maken ({}): {err}", path.display());
        return false;
    }

    let pretty = ron::ser::PrettyConfig::default();
    for scenario in default_scenarios() {
        let file_path = path.join(format!("{}.ron", scenario.id));
        if file_path.exists() {
            continue;
        }

        let serialized = match ron::ser::to_string_pretty(&scenario, pretty.clone()) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Kon scenario '{}' niet serialiseren: {err}", scenario.id);
                return false;
            }
        };

        if let Err(err) = fs::write(&file_path, serialized) {
            eprintln!("Kon scenario niet opslaan ({}): {err}", file_path.display());
            return false;
        }
    }

    true
}

fn write_default_scenarios_to_file(path: &Path) -> bool {
    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            eprintln!("Kon scenario-map niet maken ({}): {err}", parent.display());
            return false;
        }
    }

    let serialized =
        match ron::ser::to_string_pretty(&default_scenarios(), ron::ser::PrettyConfig::default()) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Kon standaardscenario's niet serialiseren: {err}");
                return false;
            }
        };

    if let Err(err) = fs::write(path, serialized) {
        eprintln!("Kon scenario-bestand niet opslaan ({}): {err}", path.display());
        return false;
    }

    true
}

fn load_scenarios_from_file(path: &Path) -> Vec<ScenarioDefinition> {
    let source = path.display().to_string();
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Kon scenario-bestand niet lezen ({}): {err}", path.display());
            return Vec::new();
        }
    };

    match ron::from_str::<Vec<ScenarioDefinition>>(&content) {
        Ok(scenarios) => return filter_valid_scenarios(scenarios, &source),
        Err(_) => {}
    }

    match ron::from_str::<ScenarioDefinition>(&content) {
        Ok(scenario) => filter_valid_scenarios(vec![scenario], &source),
        Err(err) => {
            eprintln!("Kon scenario's niet parsen ({source}): {err}");
            Vec::new()
        }
    }
}

fn load_scenarios_from_dir(path: &Path) -> Vec<ScenarioDefinition> {
    let dir_iter = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("Kon scenario-map niet lezen ({}): {err}", path.display());
            return Vec::new();
        }
    };

    let mut files = Vec::<PathBuf>::new();
    for entry in dir_iter {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if path.is_file() && is_ron_file(&path) {
            files.push(path);
        }
    }
    files.sort();

    let mut scenarios = Vec::new();
    for file in files {
        let source = file.display().to_string();
        let content = match fs::read_to_string(&file) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Kon scenario-bestand niet lezen ({}): {err}", file.display());
                continue;
            }
        };

        match ron::from_str::<ScenarioDefinition>(&content) {
            Ok(scenario) => {
                if scenario.id.trim().is_empty() || scenario.name.trim().is_empty() {
                    eprintln!("Scenario-bestand ({source}) mist id of naam");
                    continue;
                }
                scenarios.push(scenario);
            }
            Err(single_err) => match ron::from_str::<Vec<ScenarioDefinition>>(&content) {
                Ok(list) => {
                    eprintln!(
                        "Scenario-bestand ({source}) bevat een lijst; gebruik bij voorkeur 1 bestand per scenario"
                    );
                    scenarios.extend(filter_valid_scenarios(list, &source));
                }
                Err(_) => {
                    eprintln!("Kon scenario-bestand niet parsen ({source}): {single_err}");
                }
            },
        }
    }

    scenarios
}

fn load_scenario_catalog(path: &Path) -> ScenarioCatalog {
    let mut scenarios = if path.exists() {
        if path.is_dir() {
            load_scenarios_from_dir(path)
        } else {
            load_scenarios_from_file(path)
        }
    } else if is_ron_file(path) {
        if write_default_scenarios_to_file(path) {
            println!("Scenario-bestand aangemaakt: {}", path.display());
            load_scenarios_from_file(path)
        } else {
            Vec::new()
        }
    } else if write_default_scenarios_to_dir(path) {
        println!("Scenario-map aangemaakt: {}", path.display());
        load_scenarios_from_dir(path)
    } else {
        Vec::new()
    };

    if scenarios.is_empty() {
        if path.is_dir() && write_default_scenarios_to_dir(path) {
            scenarios = load_scenarios_from_dir(path);
        }
    }

    if scenarios.is_empty() {
        eprintln!("Geen geldige scenario's beschikbaar, gebruik ingebouwde fallback.");
        scenarios = default_scenarios();
    }

    ScenarioCatalog { scenarios }
}

fn load_persisted_config() -> PersistedConfig {
    let path = Path::new(CONFIG_PATH);

    let Ok(content) = fs::read_to_string(path) else {
        return PersistedConfig::default();
    };

    match ron::from_str::<PersistedConfig>(&content) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Kon config niet lezen ({}): {err}", path.display());
            PersistedConfig::default()
        }
    }
}

fn save_persisted_config(settings: &GameSettings, keybinds: &GameKeybinds) {
    let persisted = PersistedConfig {
        settings: settings.clone(),
        keybinds: PersistedKeybinds::from_runtime(keybinds),
    };

    let path = Path::new(CONFIG_PATH);
    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            eprintln!("Kon config-map niet maken ({}): {err}", parent.display());
            return;
        }
    }

    let pretty = ron::ser::PrettyConfig::default();
    let serialized = match ron::ser::to_string_pretty(&persisted, pretty) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Kon config niet serialiseren: {err}");
            return;
        }
    };

    if let Err(err) = fs::write(path, serialized) {
        eprintln!("Kon config niet opslaan ({}): {err}", path.display());
    }
}

fn action_matches_filter(action: GameAction, filter: &str) -> bool {
    if filter.is_empty() {
        return true;
    }

    action
        .label()
        .to_ascii_lowercase()
        .contains(&filter.to_ascii_lowercase())
}

fn keycode_to_filter_char(key: KeyCode) -> Option<char> {
    match key {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        KeyCode::Space => Some(' '),
        _ => None,
    }
}

fn keycodes_to_names(keys: &[KeyCode]) -> String {
    keys.iter()
        .map(|key| keycode_to_name(*key))
        .collect::<Vec<_>>()
        .join("|")
}

fn keycodes_from_names(raw: &str) -> Vec<KeyCode> {
    let mut out = Vec::new();
    for segment in raw.split('|') {
        let key_name = segment.trim();
        if key_name.is_empty() {
            continue;
        }
        if let Some(key) = keycode_from_name(key_name) {
            if !out.contains(&key) {
                out.push(key);
            }
        }
    }
    out
}

fn keycode_to_name(key: KeyCode) -> String {
    format!("{key:?}")
}

fn keycode_to_label(key: KeyCode) -> String {
    match key {
        KeyCode::KeyA => "A".into(),
        KeyCode::KeyB => "B".into(),
        KeyCode::KeyC => "C".into(),
        KeyCode::KeyD => "D".into(),
        KeyCode::KeyE => "E".into(),
        KeyCode::KeyF => "F".into(),
        KeyCode::KeyG => "G".into(),
        KeyCode::KeyH => "H".into(),
        KeyCode::KeyI => "I".into(),
        KeyCode::KeyJ => "J".into(),
        KeyCode::KeyK => "K".into(),
        KeyCode::KeyL => "L".into(),
        KeyCode::KeyM => "M".into(),
        KeyCode::KeyN => "N".into(),
        KeyCode::KeyO => "O".into(),
        KeyCode::KeyP => "P".into(),
        KeyCode::KeyQ => "Q".into(),
        KeyCode::KeyR => "R".into(),
        KeyCode::KeyS => "S".into(),
        KeyCode::KeyT => "T".into(),
        KeyCode::KeyU => "U".into(),
        KeyCode::KeyV => "V".into(),
        KeyCode::KeyW => "W".into(),
        KeyCode::KeyX => "X".into(),
        KeyCode::KeyY => "Y".into(),
        KeyCode::KeyZ => "Z".into(),
        KeyCode::Digit0 => "0".into(),
        KeyCode::Digit1 => "1".into(),
        KeyCode::Digit2 => "2".into(),
        KeyCode::Digit3 => "3".into(),
        KeyCode::Digit4 => "4".into(),
        KeyCode::Digit5 => "5".into(),
        KeyCode::Digit6 => "6".into(),
        KeyCode::Digit7 => "7".into(),
        KeyCode::Digit8 => "8".into(),
        KeyCode::Digit9 => "9".into(),
        _ => format!("{key:?}"),
    }
}

fn keycode_from_name(name: &str) -> Option<KeyCode> {
    match name {
        "KeyA" => Some(KeyCode::KeyA),
        "KeyB" => Some(KeyCode::KeyB),
        "KeyC" => Some(KeyCode::KeyC),
        "KeyD" => Some(KeyCode::KeyD),
        "KeyE" => Some(KeyCode::KeyE),
        "KeyF" => Some(KeyCode::KeyF),
        "KeyG" => Some(KeyCode::KeyG),
        "KeyH" => Some(KeyCode::KeyH),
        "KeyI" => Some(KeyCode::KeyI),
        "KeyJ" => Some(KeyCode::KeyJ),
        "KeyK" => Some(KeyCode::KeyK),
        "KeyL" => Some(KeyCode::KeyL),
        "KeyM" => Some(KeyCode::KeyM),
        "KeyN" => Some(KeyCode::KeyN),
        "KeyO" => Some(KeyCode::KeyO),
        "KeyP" => Some(KeyCode::KeyP),
        "KeyQ" => Some(KeyCode::KeyQ),
        "KeyR" => Some(KeyCode::KeyR),
        "KeyS" => Some(KeyCode::KeyS),
        "KeyT" => Some(KeyCode::KeyT),
        "KeyU" => Some(KeyCode::KeyU),
        "KeyV" => Some(KeyCode::KeyV),
        "KeyW" => Some(KeyCode::KeyW),
        "KeyX" => Some(KeyCode::KeyX),
        "KeyY" => Some(KeyCode::KeyY),
        "KeyZ" => Some(KeyCode::KeyZ),
        "Digit0" => Some(KeyCode::Digit0),
        "Digit1" => Some(KeyCode::Digit1),
        "Digit2" => Some(KeyCode::Digit2),
        "Digit3" => Some(KeyCode::Digit3),
        "Digit4" => Some(KeyCode::Digit4),
        "Digit5" => Some(KeyCode::Digit5),
        "Digit6" => Some(KeyCode::Digit6),
        "Digit7" => Some(KeyCode::Digit7),
        "Digit8" => Some(KeyCode::Digit8),
        "Digit9" => Some(KeyCode::Digit9),
        "Space" => Some(KeyCode::Space),
        "Tab" => Some(KeyCode::Tab),
        "Enter" => Some(KeyCode::Enter),
        "Backspace" => Some(KeyCode::Backspace),
        "ShiftLeft" => Some(KeyCode::ShiftLeft),
        "ShiftRight" => Some(KeyCode::ShiftRight),
        "ControlLeft" => Some(KeyCode::ControlLeft),
        "ControlRight" => Some(KeyCode::ControlRight),
        "AltLeft" => Some(KeyCode::AltLeft),
        "AltRight" => Some(KeyCode::AltRight),
        "ArrowUp" => Some(KeyCode::ArrowUp),
        "ArrowDown" => Some(KeyCode::ArrowDown),
        "ArrowLeft" => Some(KeyCode::ArrowLeft),
        "ArrowRight" => Some(KeyCode::ArrowRight),
        "Minus" => Some(KeyCode::Minus),
        "Equal" => Some(KeyCode::Equal),
        "BracketLeft" => Some(KeyCode::BracketLeft),
        "BracketRight" => Some(KeyCode::BracketRight),
        "Semicolon" => Some(KeyCode::Semicolon),
        "Quote" => Some(KeyCode::Quote),
        "Backquote" => Some(KeyCode::Backquote),
        "Backslash" => Some(KeyCode::Backslash),
        "Comma" => Some(KeyCode::Comma),
        "Period" => Some(KeyCode::Period),
        "Slash" => Some(KeyCode::Slash),
        "Escape" => Some(KeyCode::Escape),
        "Insert" => Some(KeyCode::Insert),
        "Delete" => Some(KeyCode::Delete),
        "Home" => Some(KeyCode::Home),
        "End" => Some(KeyCode::End),
        "PageUp" => Some(KeyCode::PageUp),
        "PageDown" => Some(KeyCode::PageDown),
        "F1" => Some(KeyCode::F1),
        "F2" => Some(KeyCode::F2),
        "F3" => Some(KeyCode::F3),
        "F4" => Some(KeyCode::F4),
        "F5" => Some(KeyCode::F5),
        "F6" => Some(KeyCode::F6),
        "F7" => Some(KeyCode::F7),
        "F8" => Some(KeyCode::F8),
        "F9" => Some(KeyCode::F9),
        "F10" => Some(KeyCode::F10),
        "F11" => Some(KeyCode::F11),
        "F12" => Some(KeyCode::F12),
        _ => None,
    }
}
