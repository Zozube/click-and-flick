#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::gltf::GltfMesh;
use bevy::render::view::RenderLayers;
use bevy::{
    color::palettes::basic::RED,
    audio::Volume,
    pbr::{ExtendedMaterial, MaterialExtension, OpaqueRendererMethod},
    scene::{SceneInstanceReady, SceneRoot},
    prelude::*,
    render::render_resource::*,
};
use core::time::Duration;
use rand::seq::IndexedRandom;

use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy::dev_tools::picking_debug::{DebugPickingMode, DebugPickingPlugin};

#[derive(Resource)]
struct MyTimer(Timer);

#[derive(Component, Default)]
struct Bouncer {
    spd: f32,
    pos: f32,
}

#[derive(Component)]
struct Coin {}

#[derive(Resource)]
struct AudioSamples {
    samples: Vec<Handle<AudioSource>>,
}

impl Bouncer {
    pub fn bounce(&mut self) {
        self.spd = 5.0;
    }

    pub fn update(&mut self, delta: Duration) {
        self.spd += -50.0 * delta.as_secs_f32();

        if self.spd > 10. {
            self.spd = 10.;
        }

        self.pos += self.spd * delta.as_secs_f32();

        if self.pos < 0.0 {
            self.pos = 0.0;
            self.spd = 0.0;
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            filter: "bevy_dev_tools=trace".into(), // Show picking logs trace level and up
            ..default()
        }))
        .add_plugins((MeshPickingPlugin, DebugPickingPlugin))
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(MyTimer(Timer::from_seconds(1.0, TimerMode::Once)))
        .add_systems(Startup, (setup, setup_camera, load_punches, spawn_gltf))
        .add_systems(Update, (mouse_button_input, update, rotate_coin))
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
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        RenderLayers::layer(1),
    ));
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            ..default()
        },
        Transform::from_xyz(0., 0., 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::layer(0),
    ));
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Sprite::from_image(asset_server.load("private/cave-blue.png")),
        Transform::from_xyz(0., 0., 1.),
        RenderLayers::layer(1),
    ));
    commands.spawn(Bouncer::default());
}

fn load_punches(asset_server: Res<AssetServer>, mut commands: Commands) {
    let samples: Vec<Handle<AudioSource>> = (1..=6)
        .map(|i| asset_server.load(format!("private/non-commercial/punch/{}.ogg", i)))
        .collect();

    commands.insert_resource(AudioSamples { samples });
}

fn spawn_gltf(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: light_consts::lux::OVERCAST_DAY,
            ..default()
        },
        Transform {
            rotation: Quat::from_rotation_y(45.),
            ..default()
        },
        RenderLayers::layer(0),
    ));
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: light_consts::lux::OVERCAST_DAY,
            ..default()
        },
        Transform {
            rotation: Quat::from_rotation_y(-45.),
            ..default()
        },
        RenderLayers::layer(0),
    ));

    //let obj: Handle<Mesh> = asset_server.load("private/coin.obj");

    let mesh: Handle<Mesh> = asset_server.load(
        GltfAssetLabel::Primitive {
            mesh: 0,
            primitive: 0,
        }
        .from_asset("private/coin.glb")
    );

    let new_mat = StandardMaterial {
        base_color: Color::linear_rgb(1.0, 0.75, 0.1),
        perceptual_roughness: 0.25,
        metallic: 0.96,
        reflectance: 0.96,

        opaque_render_method: OpaqueRendererMethod::Auto,
        ..Default::default()
    };

    // to position our 3d model, simply use the Transform
    // in the SceneBundle
    commands.spawn((
        Coin {},
        //SceneRoot(my_gltf),
        Mesh3d(mesh.clone()),
        MeshMaterial3d(materials.add(new_mat)),
        Transform::from_xyz(0., 0., -3.).looking_at(Vec3::ZERO, Vec3::X),
        RenderLayers::layer(0),

        //Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.))),
    ));
}

fn mouse_button_input(
    buttons: Res<ButtonInput<MouseButton>>,
    mut timer: ResMut<MyTimer>,
    mut bouncer_q: Query<&mut Bouncer>,
    mut commands: Commands,
    samples: Res<AudioSamples>,
) {
    let mut bouncer = bouncer_q.single_mut().unwrap();
    if buttons.just_pressed(MouseButton::Left) {
        timer.0.reset();
        bouncer.bounce();
        let sample = samples.samples.choose(&mut rand::rng()).unwrap();
        commands.spawn((
            AudioPlayer::new(sample.clone()),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: Volume::Linear(1.0),
                ..default()
            },
        ));
    }
}

fn update(
    time: Res<Time>,
    //mut timer: ResMut<MyTimer>,
    mut bouncer_q: Query<&mut Bouncer>,
    mut transform_q: Query<&mut Transform, With<Sprite>>,
) {
    let delta = time.delta();
    //timer.0.tick(delta);
    let mut bouncer = bouncer_q.single_mut().unwrap();
    let mut transform = transform_q.single_mut().unwrap();
    transform.scale = Vec3::splat(0.4 + bouncer.pos / 10.0);
    bouncer.update(delta);
    //println!("{}", bouncer.pos);
}

fn rotate_coin(time: Res<Time>, mut transform_q: Query<&mut Transform, With<Coin>>) {
    let delta = time.delta();

    let mut coin_t = transform_q.single_mut().unwrap();
    coin_t.rotate_y(delta.as_secs_f32() * 4.);
}
