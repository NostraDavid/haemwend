use crate::blender_model_editor::preview::{PreviewCamera, frame_camera_target_distance};
use crate::blender_model_editor::state::EditorState;
use crate::blender_model_editor::{DEFAULT_CAMERA_PITCH_DEG, DEFAULT_CAMERA_YAW_DEG};
use bevy::camera::Viewport;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow, Window};

#[derive(Resource)]
pub struct OrbitCameraState {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub min_distance: f32,
    pub max_distance: f32,
}

impl Default for OrbitCameraState {
    fn default() -> Self {
        Self {
            target: Vec3::new(0.0, 0.0, 0.5),
            distance: 4.0,
            yaw: DEFAULT_CAMERA_YAW_DEG.to_radians(),
            pitch: DEFAULT_CAMERA_PITCH_DEG.to_radians(),
            min_distance: 0.35,
            max_distance: 80.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct UiInteractionState {
    pub wants_pointer_input: bool,
    pub wants_keyboard_input: bool,
    pub side_panel_width: f32,
}

#[derive(Resource, Default)]
pub struct MouseCaptureState {
    pub active: bool,
    pub restore_position: Option<Vec2>,
}

#[derive(Clone, Copy)]
pub enum CameraPreset {
    IsoLeft,
    IsoRight,
    Front,
    Back,
    Left,
    Right,
    Top,
}

fn camera_preset_angles(preset: CameraPreset) -> (f32, f32) {
    match preset {
        CameraPreset::IsoLeft => (45.0, -45.0),
        CameraPreset::IsoRight => (-45.0, -45.0),
        CameraPreset::Front => (0.0, -8.0),
        CameraPreset::Back => (180.0, -8.0),
        CameraPreset::Left => (90.0, -8.0),
        CameraPreset::Right => (-90.0, -8.0),
        CameraPreset::Top => (0.0, -89.0),
    }
}

pub fn apply_camera_preset(
    orbit: &mut OrbitCameraState,
    state: &EditorState,
    preset: CameraPreset,
) {
    let (target, distance) = frame_camera_target_distance(state);
    let (yaw_deg, pitch_deg) = camera_preset_angles(preset);
    orbit.target = target;
    orbit.distance = distance;
    orbit.yaw = yaw_deg.to_radians();
    orbit.pitch = pitch_deg.to_radians();
}

pub fn update_camera_viewport(
    windows: Query<&Window, With<PrimaryWindow>>,
    ui_state: Res<UiInteractionState>,
    mut camera_query: Query<&mut Camera, With<PreviewCamera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let physical_width = window.physical_width();
    let physical_height = window.physical_height().max(1);
    if physical_width == 0 {
        return;
    }

    let panel_px = (ui_state.side_panel_width.max(0.0) * window.scale_factor() as f32) as u32;
    let viewport_x = panel_px.min(physical_width.saturating_sub(1));
    let viewport_width = physical_width.saturating_sub(viewport_x).max(1);

    let viewport = Some(Viewport {
        physical_position: UVec2::new(viewport_x, 0),
        physical_size: UVec2::new(viewport_width, physical_height),
        depth: 0.0..1.0,
    });

    for mut camera in &mut camera_query {
        camera.viewport = viewport.clone();
    }
}

pub fn orbit_camera_system(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    ui_state: Res<UiInteractionState>,
    mut orbit: ResMut<OrbitCameraState>,
    mut state: ResMut<EditorState>,
    mut camera_query: Query<&mut Transform, With<PreviewCamera>>,
) {
    let mouse_delta = Vec2::new(mouse_motion.delta.x, -mouse_motion.delta.y);
    let scroll_delta = mouse_scroll.delta.y;

    if state.request_center_view {
        let (target, distance) = frame_camera_target_distance(&state);
        orbit.target = target;
        orbit.distance = distance;
        state.request_center_view = false;
    }

    let pointer_in_window = windows
        .single()
        .ok()
        .and_then(|w| w.cursor_position())
        .is_some();
    let can_capture_mouse = pointer_in_window && !ui_state.wants_pointer_input;

    if can_capture_mouse {
        if mouse_buttons.pressed(MouseButton::Right) && mouse_delta.length_squared() > 0.0 {
            orbit.yaw -= mouse_delta.x * 0.006;
            orbit.pitch = (orbit.pitch + mouse_delta.y * 0.006).clamp(-1.45, 1.45);
        }

        if mouse_buttons.pressed(MouseButton::Middle) && mouse_delta.length_squared() > 0.0 {
            let forward = camera_forward(orbit.yaw, orbit.pitch);
            let mut right = forward.cross(Vec3::Z);
            if right.length_squared() < 1e-6 {
                right = Vec3::X;
            }
            right = right.normalize();
            let up = right.cross(forward).normalize_or_zero();

            let pan_scale = orbit.distance * 0.0018;
            orbit.target += (-mouse_delta.x * right + mouse_delta.y * up) * pan_scale;
        }

        if scroll_delta.abs() > f32::EPSILON {
            let zoom_factor = (1.0 - scroll_delta * 0.10).clamp(0.2, 5.0);
            orbit.distance =
                (orbit.distance * zoom_factor).clamp(orbit.min_distance, orbit.max_distance);
        }
    }

    let forward = camera_forward(orbit.yaw, orbit.pitch);
    let camera_position = orbit.target - forward * orbit.distance;

    for mut transform in &mut camera_query {
        *transform = Transform::from_translation(camera_position).looking_at(orbit.target, Vec3::Z);
    }
}

pub fn sync_mouse_capture(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    ui_state: Res<UiInteractionState>,
    mut capture_state: ResMut<MouseCaptureState>,
    mut window_query: Query<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    let Ok((mut window, mut cursor_options)) = window_query.single_mut() else {
        return;
    };

    let interaction_pressed =
        mouse_buttons.pressed(MouseButton::Right) || mouse_buttons.pressed(MouseButton::Middle);
    let pointer_in_window = window.cursor_position().is_some();
    let should_capture =
        window.focused && interaction_pressed && pointer_in_window && !ui_state.wants_pointer_input;

    if should_capture {
        if !capture_state.active {
            capture_state.restore_position = window.cursor_position();
            capture_state.active = true;
        }
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    } else {
        if capture_state.active {
            if let Some(pos) = capture_state.restore_position.take() {
                window.set_cursor_position(Some(pos));
            }
        }
        capture_state.active = false;
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
}

fn camera_forward(yaw: f32, pitch: f32) -> Vec3 {
    Vec3::new(
        yaw.cos() * pitch.cos(),
        yaw.sin() * pitch.cos(),
        pitch.sin(),
    )
    .normalize_or_zero()
}
