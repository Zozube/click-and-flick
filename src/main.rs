#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::prelude::*;
use bevy::audio::Volume;
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
        .insert_resource(MyTimer(Timer::from_seconds(1.0, TimerMode::Once)))
        .add_systems(Startup, (setup, setup_camera, load_punches))
        .add_systems(Update, (mouse_button_input, update))
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
    commands.spawn(Camera2d);
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Sprite::from_image(
            asset_server.load("private/cave-blue.png"),
        ),
        Transform::from_xyz(0., 0., 0.)
    ));
    commands.spawn(Bouncer::default());
}

fn load_punches(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    ) {
    let samples: Vec<Handle<AudioSource>> = (1..=6)
        .map(|i| asset_server.load(format!("private/non-commercial/punch/{}.ogg", i)))
        .collect();

    commands.insert_resource(AudioSamples { samples });
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
}
