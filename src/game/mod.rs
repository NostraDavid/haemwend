use bevy::app::AppExit;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::pbr::wireframe::Wireframe;
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PresentMode, PrimaryWindow, WindowResolution};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

mod components;
mod gameplay_physics;
mod io_and_scenarios;
mod settings;
mod types;
mod ui_and_flow;

use components::*;
use gameplay_physics::*;
use io_and_scenarios::*;
use settings::*;
use types::*;
use ui_and_flow::*;

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
    let initial_debug = persisted.debug;

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
            WireframePlugin::default(),
        ))
        .insert_resource(initial_settings)
        .insert_resource(initial_keybinds)
        .insert_resource(initial_debug)
        .insert_resource(GameFlowState {
            in_game: false,
            pending_scenario,
        })
        .insert_resource(scenario_catalog)
        .insert_resource(MenuState::default())
        .insert_resource(MouseLookCaptureState::default())
        .insert_resource(WorldCollisionGrid::default())
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
        .add_systems(Update, draw_debug_geometry)
        .run();
}
