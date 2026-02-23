use bevy::prelude::Resource;
use bevy::window::{MonitorSelection, VideoModeSelection, WindowMode};
use serde::{Deserialize, Serialize};

pub(super) const RESOLUTION_OPTIONS: &[(u32, u32)] = &[
    (1280, 720),
    (1600, 900),
    (1920, 1080),
    (2560, 1440),
    (3440, 1440),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum DisplayModeSetting {
    Windowed,
    FullscreenWindowed,
    FullscreenExclusive,
}

impl DisplayModeSetting {
    pub(super) fn next(self) -> Self {
        match self {
            Self::Windowed => Self::FullscreenWindowed,
            Self::FullscreenWindowed => Self::FullscreenExclusive,
            Self::FullscreenExclusive => Self::Windowed,
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Windowed => "Windowed",
            Self::FullscreenWindowed => "Fullscreen Windowed",
            Self::FullscreenExclusive => "Fullscreen",
        }
    }

    pub(super) fn to_window_mode(self) -> WindowMode {
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
pub(super) enum ShadowModeSetting {
    Blob,
    Stencil,
}

impl ShadowModeSetting {
    pub(super) fn next(self) -> Self {
        match self {
            Self::Blob => Self::Stencil,
            Self::Stencil => Self::Blob,
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Blob => "Blob",
            Self::Stencil => "Stencil",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum FogCurveSetting {
    Linear,
    Exponential,
    ExponentialSquared,
    Atmospheric,
}

impl FogCurveSetting {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::Exponential => "Exponential",
            Self::ExponentialSquared => "Exponential Squared",
            Self::Atmospheric => "Atmospheric",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum FogAnchorSetting {
    Camera,
    Character,
}

impl FogAnchorSetting {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Camera => "Camera",
            Self::Character => "Character",
        }
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct GameSettings {
    pub(super) display_mode: DisplayModeSetting,
    pub(super) resolution_width: u32,
    pub(super) resolution_height: u32,
    pub(super) msaa_enabled: bool,
    pub(super) shadow_mode: ShadowModeSetting,
    pub(super) foot_support_max_drop: f32,
    pub(super) foot_support_max_rise: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            display_mode: DisplayModeSetting::Windowed,
            resolution_width: 1920,
            resolution_height: 1080,
            msaa_enabled: true,
            shadow_mode: ShadowModeSetting::Blob,
            foot_support_max_drop: 0.45,
            foot_support_max_rise: 0.42,
        }
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct DebugSettings {
    pub(super) show_performance_overlay: bool,
    pub(super) show_baked_shadows: bool,
    pub(super) show_fog: bool,
    pub(super) fog_anchor: FogAnchorSetting,
    pub(super) fog_curve: FogCurveSetting,
    pub(super) fog_start: f32,
    pub(super) fog_end: f32,
    pub(super) fog_density: f32,
    pub(super) fog_use_visibility: bool,
    pub(super) fog_visibility_distance: f32,
    pub(super) fog_visibility_transmittance: f32,
    pub(super) fog_clear_offset: f32,
    pub(super) fog_color: (f32, f32, f32),
    pub(super) fog_opacity: f32,
    pub(super) fog_hide_geometry: bool,
    // Legacy field kept for backwards compatibility with older persisted configs.
    pub(super) fog_curvature: f32,
    pub(super) show_collision_shapes: bool,
    pub(super) show_animation_debug: bool,
    pub(super) show_wireframe: bool,
    pub(super) show_world_axes: bool,
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            show_performance_overlay: true,
            show_baked_shadows: true,
            show_fog: true,
            fog_anchor: FogAnchorSetting::Character,
            fog_curve: FogCurveSetting::ExponentialSquared,
            fog_start: 22.0,
            fog_end: 78.0,
            fog_density: 0.0125,
            fog_use_visibility: true,
            fog_visibility_distance: 78.0,
            fog_visibility_transmittance: 0.02,
            fog_clear_offset: 0.0,
            fog_color: (0.62, 0.72, 0.84),
            fog_opacity: 1.0,
            fog_hide_geometry: false,
            fog_curvature: 1.0,
            show_collision_shapes: false,
            show_animation_debug: false,
            show_wireframe: false,
            show_world_axes: false,
        }
    }
}
