#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod main_menu;
mod mine_plugin;
mod states;

use bevy::prelude::*;

use bevy::dev_tools::picking_debug::{DebugPickingMode, DebugPickingPlugin};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::mine_plugin::MinePlugin;
use crate::states::{AppState, GameState};

fn main() {
    App::new()
        // Plugins
        .add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            filter: "bevy_dev_tools=trace".into(), // Show picking logs trace level and up
            ..default()
        }))
        .add_plugins((MeshPickingPlugin, DebugPickingPlugin))
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(MinePlugin)
        .add_systems(
            PreUpdate,
            (|mut mode: ResMut<DebugPickingMode>| {
                *mode = match *mode {
                    DebugPickingMode::Disabled => DebugPickingMode::Normal,
                    DebugPickingMode::Normal => DebugPickingMode::Noisy,
                    DebugPickingMode::Noisy => DebugPickingMode::Disabled,
                }
            })
            .distributive_run_if(bevy::input::common_conditions::input_just_pressed(
                KeyCode::F3,
            )),
        )
        // States
        .init_state::<GameState>()
        .init_state::<AppState>()
        .run();
}
