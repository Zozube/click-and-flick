#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod main_menu;
mod map;
mod mine_plugin;
mod scene_change_plugin;
mod states;
mod util;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_pancam::PanCamPlugin;
use bevy_simple_screen_boxing::CameraBoxingPlugin;

use bevy::dev_tools::picking_debug::{DebugPickingMode, DebugPickingPlugin};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use scene_change_plugin::SceneChangePlugin;

use crate::map::MapPlugin;
use crate::mine_plugin::MinePlugin;
use crate::states::{AppState, GameState};

fn main() {
    App::new()
        // Plugins
        .add_plugins(
            DefaultPlugins
                .set(bevy::log::LogPlugin {
                    filter: "bevy_dev_tools=trace".into(), // Show picking logs trace level and up
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        //.configure_sets(Update, GameLogic)
        //.configure_sets(Startup, GameLogic)
        .add_plugins(PanCamPlugin::default())
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(CameraBoxingPlugin)
        .add_plugins((MeshPickingPlugin, DebugPickingPlugin))
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(MinePlugin)
        .add_plugins(MapPlugin)
        .add_plugins(SceneChangePlugin)
        //.configure_sets(Update, GameLogic.run_if(in_state(GameState::Mine)))
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
        .add_systems(
            Update,
            (
                /* gizmos, */ boxes, /*upd_letterbox.run_if(on_event::<WindowResized>)*/
            ),
        )
        .run();
}

fn boxes(
    mut gizmo: Gizmos,
    q_sprite: Query<(&Sprite, &Transform)>,
    images: Res<Assets<Image>>,
    debug_mode: Res<DebugPickingMode>,
) {
    if *debug_mode == DebugPickingMode::Disabled {
        return;
    }

    for (sprite, transform) in q_sprite.iter() {
        if let Some(texture) = images.get(sprite.image.id()) {
            let texture_size = texture.size();
            let scale = transform.scale.truncate();
            let size = texture_size.as_vec2() * scale;
            //println!("{:?}", size);
            let half_size = size * 0.5;
            let pos = transform.translation.truncate();

            let min = pos - half_size;
            let max = pos + half_size;

            gizmo.cross_2d((min + max) / 2., 12., Color::linear_rgb(1., 1., 1.));
            gizmo.cross_2d(min, 12., Color::linear_rgb(0., 1., 1.));
            gizmo.cross_2d(max, 12., Color::linear_rgb(0., 1., 1.));
            gizmo.rect_2d(
                Isometry2d::from_translation((max + min) / 2.),
                max - min,
                Color::linear_rgb(1., 0., 1.),
            );
        }
    }
}

fn gizmos(mut gizmo: Gizmos, q_projection: Query<&Projection, With<Camera2d>>) {
    let projetcion = match q_projection.single() {
        Ok(v) => v,
        Err(_) => return,
    };

    if let Projection::Orthographic(p) = projetcion {
        let size = p.area.max - p.area.min;
        gizmo.rect_2d(Isometry2d::IDENTITY, size, Color::linear_rgb(1., 1., 0.));
        gizmo.line_2d(size / 2., -size / 2., Color::linear_rgb(1., 1., 0.));
        gizmo.line_2d(
            Vec2 { x: -size.x, ..size } / 2.,
            Vec2 { y: -size.y, ..size } / 2.,
            Color::linear_rgb(1., 1., 0.),
        );
    }
}
