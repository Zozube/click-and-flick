#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod main_menu;
mod mine_plugin;
mod states;

use avian3d::prelude::*;
use bevy::math::AspectRatio;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy_simple_screen_boxing::CameraBox;
use bevy_simple_screen_boxing::CameraBoxingPlugin;
use std::error::Error;

use bevy::dev_tools::picking_debug::{DebugPickingMode, DebugPickingPlugin};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::mine_plugin::MinePlugin;
use crate::states::{AppState, GameState};

#[derive(Resource, Debug, Clone)]
pub struct Letterbox {
    pub viewport: bevy::render::camera::Viewport,
    pub region: Rect,
}

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
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(CameraBoxingPlugin)
        .add_plugins((MeshPickingPlugin, DebugPickingPlugin))
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(ClearColor(Color::BLACK))
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
        .insert_resource(Letterbox {
            region: Rect::EMPTY,
            viewport: bevy::render::camera::Viewport { ..default() },
        })
        .add_systems(
            Update,
            (
                gizmos, boxes, /*upd_letterbox.run_if(on_event::<WindowResized>)*/
            ),
        )
        .add_systems(Startup, (/*upd_letterbox, */setup))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        CameraBox::ResolutionIntegerScale {
            resolution: Vec2::new(1920., 1080.),
            allow_imperfect_aspect_ratios: true,
        },
        RenderLayers::layer(0),
        Projection::Orthographic(OrthographicProjection {
            //viewport_origin: Vec2::ZERO,
            scaling_mode: bevy::render::camera::ScalingMode::Fixed {
                width: 1920.,
                height: 1080.,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            ..default()
        },
        CameraBox::ResolutionIntegerScale {
            resolution: Vec2::new(1920., 1080.),
            allow_imperfect_aspect_ratios: true,
        },
        Transform::from_xyz(0., 0., 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::layer(1),
    ));

    //commands.spawn((
    //    Sprite::from_image(asset_server.load("private/rock-layer0.png")),
    //    Transform::from_xyz(-200., 0., 100.).with_scale(Vec3::splat(0.33)),
    //));
}

fn parse_aspect_ratio(ratio: &str) -> Result<AspectRatio, Box<dyn Error>> {
    let mut splits = ratio.split(":");
    let format_err = "Must be 2 numbers formatted as x:y";
    let x = splits.next().ok_or(format_err)?.trim().parse::<f32>()?;
    let y = splits.next().ok_or(format_err)?.trim().parse::<f32>()?;
    return AspectRatio::try_new(x, y).map_err(|e| Box::new(e) as Box<dyn Error>);
}

fn get_common_aspect_ratio(target: &AspectRatio, other: &[&AspectRatio]) -> f32 {
    let ar = target.ratio();
    let (min_ar, max_ar) = other
        .iter()
        .map(|d| d.ratio())
        .fold(None, |acc, val| match acc {
            None => Some((val, val)),
            Some((min, max)) => Some((min.min(val), max.max(val))),
        })
        .unwrap();
    ar
}

fn calculate_letterbox(window: &Window) -> Letterbox {
    let ww = window.width();
    let wh = window.height();
    let wr = ww / wh;

    let primary_ar = parse_aspect_ratio("16:9").unwrap();
    let min_ar = parse_aspect_ratio("16:10").unwrap();
    let max_ar = parse_aspect_ratio("20:9").unwrap();

    let min_r = min_ar.ratio();
    let max_r = max_ar.ratio();

    //let pr = min_r * max_r;
    let pr = primary_ar.ratio();

    let size = if wr > pr {
        Vec2 { x: wh * pr, y: wh }
    } else {
        Vec2 { x: ww, y: ww / pr }
    };

    // ViewPort
    let vp_left = ((ww - size.x) / 2.0).round() as u32;
    let vp_bottom = ((wh - size.y) / 2.0).round() as u32;

    Letterbox {
        region: Rect {
            min: Vec2 {
                x: -size.x / ww,
                y: -size.y / wh,
            },
            max: Vec2 {
                x: size.x / ww,
                y: size.y / wh,
            },
        },
        viewport: bevy::render::camera::Viewport {
            physical_position: UVec2::new(vp_left, vp_bottom),
            physical_size: UVec2::new(size.x.round() as u32, size.y.round() as u32),
            depth: 0.0..1.0,
        },
    }
}

fn upd_letterbox(
    windows: Query<&Window>,
    mut letterbox: ResMut<Letterbox>,
    mut cameras: Query<&mut Camera>,
) {
    let window = windows.single().expect("Multiple windows present");
    let lb = calculate_letterbox(window);
    letterbox.viewport = lb.viewport.clone();
    letterbox.region = lb.region;

    for mut camera in cameras.iter_mut() {
        camera.viewport = Some(lb.viewport.clone());
        //println!("{:?}", camera)
    }

    //for mut bg in backgrounds.iter_mut() {
    //    bg.custom_size = Some(lb.viewport.physical_size.as_vec2());
    //}
}

fn boxes(mut gizmo: Gizmos, q_sprite: Query<(&Sprite, &Transform)>, images: Res<Assets<Image>>) {
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
    let projetcion = q_projection.single().unwrap();
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
