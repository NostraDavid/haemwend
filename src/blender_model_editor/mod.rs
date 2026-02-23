pub mod camera;
pub mod editor;
pub mod jobs;
pub mod model;
pub mod preview;
pub mod state;
pub mod ui;

pub const MODELS_CONFIG_PATH: &str = "config/blender_ai_models.ron";
pub const LIVE_REPORT_PATH: &str = "assets/blender_ai/_artifacts/live_report.json";
pub const PRESETS_PATH: &str = "assets/blender_ai/_artifacts/editor_presets.ron";
pub const GRID_EXTENT_METERS: i32 = 20;
pub const GRID_MAJOR_STEP_METERS: i32 = 5;
pub const DEFAULT_CAMERA_YAW_DEG: f32 = 45.0;
pub const DEFAULT_CAMERA_PITCH_DEG: f32 = -45.0;
