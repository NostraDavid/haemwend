use super::io_and_scenarios::{keycode_to_label, keycodes_from_names, keycodes_to_names};
use super::settings::GameSettings;
use bevy::prelude::{ButtonInput, KeyCode, Resource, Vec3};
use serde::{Deserialize, Serialize};

pub(super) const CONFIG_PATH: &str = "config/game_config.ron";
pub(super) const SCENARIOS_PATH_DEFAULT: &str = "config/scenarios";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ScenarioDefinition {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) description: String,
    pub(super) ground_extent: f32,
    pub(super) crate_grid_radius: i32,
    pub(super) crate_spacing: f32,
    pub(super) crate_pattern_mod: i32,
    pub(super) wall_count: i32,
    pub(super) wall_spacing: f32,
    pub(super) wall_z: f32,
    pub(super) tower_z: f32,
    pub(super) sun_position: [f32; 3],
}

impl ScenarioDefinition {
    pub(super) fn sun_vec3(&self) -> Vec3 {
        Vec3::new(
            self.sun_position[0],
            self.sun_position[1],
            self.sun_position[2],
        )
    }
}

#[derive(Resource, Debug, Clone)]
pub(super) struct ScenarioCatalog {
    pub(super) scenarios: Vec<ScenarioDefinition>,
}

impl ScenarioCatalog {
    pub(super) fn index_by_id(&self, id: &str) -> Option<usize> {
        self.scenarios.iter().position(|scenario| scenario.id == id)
    }
}

#[derive(Debug, Clone)]
pub(super) struct CliOptions {
    pub(super) scenario_id: Option<String>,
    pub(super) scenarios_path: String,
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
pub(super) struct GameFlowState {
    pub(super) in_game: bool,
    pub(super) pending_scenario: Option<usize>,
}

impl Default for GameFlowState {
    fn default() -> Self {
        Self {
            in_game: false,
            pending_scenario: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum GameAction {
    MoveForward,
    MoveBackward,
    StrafeLeft,
    StrafeRight,
    TurnLeft,
    TurnRight,
    Sprint,
    Jump,
}

pub(super) const ACTION_ORDER: [GameAction; 8] = [
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
    pub(super) fn label(self) -> &'static str {
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
pub(super) struct GameKeybinds {
    pub(super) move_forward: Vec<KeyCode>,
    pub(super) move_backward: Vec<KeyCode>,
    pub(super) strafe_left: Vec<KeyCode>,
    pub(super) strafe_right: Vec<KeyCode>,
    pub(super) turn_left: Vec<KeyCode>,
    pub(super) turn_right: Vec<KeyCode>,
    pub(super) sprint: Vec<KeyCode>,
    pub(super) jump: Vec<KeyCode>,
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
    pub(super) fn keys_for(&self, action: GameAction) -> &[KeyCode] {
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

    pub(super) fn keys_for_mut(&mut self, action: GameAction) -> &mut Vec<KeyCode> {
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

    pub(super) fn action_pressed(&self, input: &ButtonInput<KeyCode>, action: GameAction) -> bool {
        self.keys_for(action).iter().any(|key| input.pressed(*key))
    }

    pub(super) fn action_just_pressed(
        &self,
        input: &ButtonInput<KeyCode>,
        action: GameAction,
    ) -> bool {
        self.keys_for(action)
            .iter()
            .any(|key| input.just_pressed(*key))
    }

    pub(super) fn add_key(&mut self, action: GameAction, key: KeyCode) -> bool {
        let keys = self.keys_for_mut(action);
        if keys.contains(&key) {
            return false;
        }

        keys.push(key);
        true
    }

    pub(super) fn remove_key(&mut self, action: GameAction, key: KeyCode) -> bool {
        let keys = self.keys_for_mut(action);
        if keys.len() <= 1 {
            return false;
        }

        let old_len = keys.len();
        keys.retain(|k| *k != key);
        old_len != keys.len()
    }

    pub(super) fn has_key(&self, action: GameAction, key: KeyCode) -> bool {
        self.keys_for(action).contains(&key)
    }

    pub(super) fn display_keys(&self, action: GameAction) -> String {
        self.keys_for(action)
            .iter()
            .map(|key| keycode_to_label(*key))
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub(super) fn ensure_non_empty(&mut self) {
        for action in ACTION_ORDER {
            if self.keys_for(action).is_empty() {
                let fallback = GameKeybinds::default();
                self.keys_for_mut(action).push(fallback.keys_for(action)[0]);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct PersistedKeybinds {
    pub(super) move_forward: String,
    pub(super) move_backward: String,
    pub(super) strafe_left: String,
    pub(super) strafe_right: String,
    pub(super) turn_left: String,
    pub(super) turn_right: String,
    pub(super) sprint: String,
    pub(super) jump: String,
}

impl Default for PersistedKeybinds {
    fn default() -> Self {
        Self::from_runtime(&GameKeybinds::default())
    }
}

impl PersistedKeybinds {
    pub(super) fn from_runtime(bindings: &GameKeybinds) -> Self {
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

    pub(super) fn to_runtime(&self) -> GameKeybinds {
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
pub(super) struct PersistedConfig {
    pub(super) settings: GameSettings,
    pub(super) keybinds: PersistedKeybinds,
}

impl Default for PersistedConfig {
    fn default() -> Self {
        Self {
            settings: GameSettings::default(),
            keybinds: PersistedKeybinds::default(),
        }
    }
}
