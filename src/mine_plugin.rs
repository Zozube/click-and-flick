use crate::states::GameState;

use bevy::render::view::RenderLayers;
use bevy::{audio::Volume, pbr::OpaqueRendererMethod, prelude::*};
use core::time::Duration;
use rand::{seq::SliceRandom, thread_rng};

#[derive(Component)]
struct Coin {}

#[derive(Resource)]
struct AudioSamples {
    samples: Vec<Handle<AudioSource>>,
}

#[derive(Resource)]
struct Ambient(Handle<AudioSource>);

#[derive(Component, Default)]
pub struct Bouncer {
    spd: f32,
    pos: f32,
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

pub struct MinePlugin;

impl Plugin for MinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Mine),
            (load_audio, (setup, setup_camera, spawn_gltf)).chain(),
        )
        .add_systems(
            Update,
            (mouse_button_input, update, rotate_coin).run_if(in_state(GameState::Mine)),
        );
    }
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, sample: Res<Ambient>) {
    commands.spawn((
        Sprite::from_image(asset_server.load("private/cave-blue.png")),
        Transform::from_xyz(0., 0., 1.),
        RenderLayers::layer(1),
    ));
    commands.spawn(Bouncer::default());
    commands.spawn((
        AudioPlayer::new(sample.0.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: Volume::Linear(0.75),
            ..default()
        },
    ));
}

fn load_audio(asset_server: Res<AssetServer>, mut commands: Commands) {
    let samples: Vec<Handle<AudioSource>> = (1..=6)
        .map(|i| asset_server.load(format!("private/non-commercial/punch/{}.ogg", i)))
        .collect();

    commands.insert_resource(Ambient(
        asset_server.load("private/non-commercial/ambient/music.ogg"),
    ));
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
        .from_asset("private/coin.glb"),
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
    mut bouncer_q: Query<&mut Bouncer>,
    mut commands: Commands,
    samples: Res<AudioSamples>,
) {
    let mut bouncer = bouncer_q.single_mut().unwrap();
    if buttons.just_pressed(MouseButton::Left) {
        bouncer.bounce();
        let sample = samples.samples.choose(&mut thread_rng()).unwrap();
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
