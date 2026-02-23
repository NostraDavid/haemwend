use super::types::GameAction;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MenuScreen {
    Main,
    Settings,
    Debug,
    Keybinds,
    ExitConfirm,
}

#[derive(Resource, Debug)]
pub(super) struct MenuState {
    pub(super) open: bool,
    pub(super) screen: MenuScreen,
    pub(super) awaiting_rebind: Option<GameAction>,
    pub(super) keybind_filter: String,
    pub(super) dirty: bool,
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
pub(super) struct MouseLookCaptureState {
    pub(super) active: bool,
    pub(super) restore_position: Option<Vec2>,
}

#[derive(Component)]
pub(super) struct Player {
    pub(super) walk_speed: f32,
    pub(super) sprint_speed: f32,
    pub(super) turn_speed: f32,
    pub(super) jump_speed: f32,
    pub(super) gravity: f32,
}

#[derive(Component)]
pub(super) struct ThirdPersonCameraRig {
    pub(super) yaw: f32,
    pub(super) pitch: f32,
    pub(super) look_sensitivity: f32,
    pub(super) zoom_sensitivity: f32,
    pub(super) distance: f32,
    pub(super) min_distance: f32,
    pub(super) max_distance: f32,
    pub(super) height: f32,
    pub(super) focus_height: f32,
}

#[derive(Component)]
pub(super) struct PerformanceOverlayText;

#[derive(Component)]
pub(super) struct PlayerBlobShadow;

#[derive(Component)]
pub(super) struct BakedShadow;

#[derive(Component)]
pub(super) struct MenuRoot;

#[derive(Component)]
pub(super) struct InGameEntity;

#[derive(Component)]
pub(super) struct StartMenuRoot;

#[derive(Component)]
pub(super) struct StartMenuCamera;

#[derive(Component, Clone, Copy)]
pub(super) struct MenuButton(pub(super) MenuButtonAction);

#[derive(Component, Clone, Copy)]
pub(super) struct StartMenuButton(pub(super) StartMenuButtonAction);

#[derive(Clone, Copy)]
pub(super) enum StartMenuButtonAction {
    StartScenario(usize),
    ExitGame,
}

#[derive(Clone, Copy)]
pub(super) enum MenuButtonAction {
    Resume,
    OpenSettings,
    OpenDebug,
    OpenKeybinds,
    OpenExitConfirm,
    BackMain,
    ExitNow,
    CycleDisplayMode,
    CycleResolution,
    ToggleMsaa,
    ToggleShadowMode,
    TogglePerformanceOverlay,
    ToggleBakedShadows,
    ToggleFog,
    StartRebind(GameAction),
    ClearKeybindFilter,
}

#[derive(Component, Clone, Copy)]
pub(super) struct PlayerCollider {
    pub(super) half_extents: Vec3,
}

#[derive(Component, Clone, Copy)]
pub(super) struct WorldCollider {
    pub(super) half_extents: Vec3,
}

#[derive(Component, Default)]
pub(super) struct PlayerKinematics {
    pub(super) vertical_velocity: f32,
    pub(super) grounded: bool,
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
