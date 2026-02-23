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

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub(super) struct GameSettings {
    pub(super) display_mode: DisplayModeSetting,
    pub(super) resolution_width: u32,
    pub(super) resolution_height: u32,
    pub(super) msaa_enabled: bool,
    pub(super) shadow_mode: ShadowModeSetting,
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
