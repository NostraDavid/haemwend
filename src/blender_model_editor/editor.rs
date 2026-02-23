use crate::blender_model_editor::camera::{
    MouseCaptureState, OrbitCameraState, UiInteractionState, orbit_camera_system,
    sync_mouse_capture, update_camera_viewport,
};
use crate::blender_model_editor::jobs::{JobQueue, poll_finished_jobs};
use crate::blender_model_editor::preview::{
    apply_live_preview, draw_grid_system, queue_initial_preview, setup_preview_scene,
};
use crate::blender_model_editor::state::load_initial_state;
use crate::blender_model_editor::ui::ui_system;
use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowPlugin};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

pub fn run() {
    App::new()
        .insert_resource(load_initial_state())
        .insert_resource(OrbitCameraState::default())
        .insert_resource(UiInteractionState::default())
        .insert_resource(MouseCaptureState::default())
        .insert_non_send_resource(JobQueue::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Blender Model Editor".to_string(),
                resolution: (1400, 900).into(),
                present_mode: PresentMode::AutoVsync,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, setup_preview_scene)
        .add_systems(Startup, queue_initial_preview)
        .add_systems(Update, apply_live_preview)
        .add_systems(Update, update_camera_viewport)
        .add_systems(Update, sync_mouse_capture)
        .add_systems(Update, orbit_camera_system)
        .add_systems(Update, draw_grid_system)
        .add_systems(Update, poll_finished_jobs)
        .add_systems(EguiPrimaryContextPass, ui_system)
        .run();
}
