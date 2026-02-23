use bevy::app::AppExit;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::window::{
    CursorGrabMode, CursorOptions, MonitorSelection, PresentMode, PrimaryWindow,
    VideoModeSelection, WindowMode, WindowResolution,
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
                self.keys_for_mut(action).push(fallback.keys_for(action)[0]);
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

pub fn run() {
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
        .add_systems(
            Update,
            (handle_start_menu_buttons, load_pending_scenario).chain(),
        )
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

mod gameplay_physics;
mod io_and_scenarios;
mod ui_and_flow;

use gameplay_physics::*;
use io_and_scenarios::*;
use ui_and_flow::*;
