use super::types::GameAction;
use bevy::prelude::*;
use std::collections::HashMap;

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
pub(super) struct StairSteepnessLabel;

#[derive(Component)]
pub(super) struct PlayerVisualPart;

#[derive(Component)]
pub(super) struct StartMenuRoot;

#[derive(Component)]
pub(super) struct StartMenuCamera;

#[derive(Component)]
pub(super) struct ProceduralHumanVisualRoot;

#[derive(Component)]
pub(super) struct ProceduralHumanAnimState {
    pub(super) phase: f32,
    pub(super) smoothed_speed: f32,
    pub(super) last_position: Vec3,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum LimbSide {
    Left,
    Right,
}

#[derive(Component)]
pub(super) struct HumanLegHip {
    pub(super) side: LimbSide,
    pub(super) base_local: Vec3,
    pub(super) upper_len: f32,
    pub(super) lower_len: f32,
    pub(super) ankle_height: f32,
}

#[derive(Component)]
pub(super) struct HumanArmPivot {
    pub(super) side: LimbSide,
    pub(super) base_local: Vec3,
}

#[derive(Component)]
pub(super) struct HumanHead {
    pub(super) base_local: Vec3,
    pub(super) max_yaw: f32,
    pub(super) max_pitch_up: f32,
    pub(super) max_pitch_down: f32,
}

#[derive(Component)]
pub(super) struct HumanLegKnee;

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
    ToggleCollisionShapes,
    ToggleWireframe,
    ToggleWorldAxes,
    StartRebind(GameAction),
    ClearKeybindFilter,
}

#[derive(Component, Clone, Copy)]
pub(super) struct PlayerCollider {
    pub(super) radius: f32,
    pub(super) half_height: f32,
}

#[derive(Component, Clone, Copy)]
pub(super) struct WorldCollider {
    pub(super) half_extents: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct StaticCollider {
    pub(super) center: Vec3,
    pub(super) half_extents: Vec3,
}

#[derive(Resource, Debug)]
pub(super) struct WorldCollisionGrid {
    pub(super) cell_size: f32,
    pub(super) cells: HashMap<IVec2, Vec<StaticCollider>>,
}

impl Default for WorldCollisionGrid {
    fn default() -> Self {
        Self {
            cell_size: 4.0,
            cells: HashMap::new(),
        }
    }
}

impl WorldCollisionGrid {
    pub(super) fn from_colliders(colliders: Vec<StaticCollider>, cell_size: f32) -> Self {
        let mut grid = Self {
            cell_size: cell_size.max(0.25),
            cells: HashMap::new(),
        };

        for collider in colliders {
            let min_x =
                ((collider.center.x - collider.half_extents.x) / grid.cell_size).floor() as i32;
            let max_x =
                ((collider.center.x + collider.half_extents.x) / grid.cell_size).floor() as i32;
            let min_z =
                ((collider.center.z - collider.half_extents.z) / grid.cell_size).floor() as i32;
            let max_z =
                ((collider.center.z + collider.half_extents.z) / grid.cell_size).floor() as i32;

            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    grid.cells
                        .entry(IVec2::new(x, z))
                        .or_default()
                        .push(collider);
                }
            }
        }

        grid
    }

    pub(super) fn query_nearby(
        &self,
        center: Vec3,
        radius: f32,
        mut visit: impl FnMut(StaticCollider),
    ) {
        if self.cells.is_empty() {
            return;
        }

        let min_x = ((center.x - radius) / self.cell_size).floor() as i32;
        let max_x = ((center.x + radius) / self.cell_size).floor() as i32;
        let min_z = ((center.z - radius) / self.cell_size).floor() as i32;
        let max_z = ((center.z + radius) / self.cell_size).floor() as i32;

        for x in min_x..=max_x {
            for z in min_z..=max_z {
                if let Some(cell_colliders) = self.cells.get(&IVec2::new(x, z)) {
                    for collider in cell_colliders {
                        visit(*collider);
                    }
                }
            }
        }
    }
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

impl ProceduralHumanAnimState {
    pub(super) fn from_position(position: Vec3) -> Self {
        Self {
            phase: 0.0,
            smoothed_speed: 0.0,
            last_position: position,
        }
    }
}
