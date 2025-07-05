use crate::states::{GameLogic, GameState};

use avian3d::prelude::*;
use bevy::asset::AssetPath;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy::{audio::Volume, pbr::OpaqueRendererMethod, prelude::*};
use core::time::Duration;
use rand::Rng;
use rand::{seq::SliceRandom, thread_rng};
use std::ops::{Deref, DerefMut};

#[derive(Component)]
struct MineSceneTag;

#[derive(Component)]
struct Coin {}

#[derive(Component)]
pub struct BackgroundImg;

#[derive(Component, Clone, Copy)]
pub struct OriginalTransform(Transform);

impl Deref for OriginalTransform {
    type Target = Transform;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OriginalTransform {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Component)]
pub struct Health(f32);

impl Health {
    pub fn hit(&mut self, value: f32) {
        self.0 -= value;
    }
}

#[derive(Component)]
#[require(Health(100.))]
pub struct Rock;

#[derive(Resource)]
struct AudioSamples {
    samples: Vec<Handle<AudioSource>>,
}

#[derive(Resource)]
struct Ambient(Handle<AudioSource>);

#[derive(Resource)]
struct MoneySpill(Handle<AudioSource>);

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

fn mesh_coin_path() -> AssetPath<'static> {
    GltfAssetLabel::Primitive {
        mesh: 0,
        primitive: 0,
    }
    .from_asset("private/coin.glb")
}

#[derive(Resource)]
struct MyMaterials {
    coin: Handle<StandardMaterial>,
}

#[derive(Resource)]
struct MyHandles {
    coin: Handle<Mesh>,
}

pub struct MinePlugin;

impl Plugin for MinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Mine),
            (load_audio, (setup, setup_camera, load_gltf)).chain(),
        )
        .add_systems(
            Update,
            (mouse_button_input, update, clean_dead).in_set(GameLogic),
        );
    }
}

fn setup_camera(mut commands: Commands) {
    //commands.spawn((
    //    Camera2d,
    //    Camera {
    //        order: 0,
    //        ..default()
    //    },
    //    RenderLayers::layer(1),
    //    Projection::Orthographic(OrthographicProjection {
    //        //viewport_origin: Vec2::ZERO,
    //        scaling_mode: bevy::render::camera::ScalingMode::Fixed {
    //            width: 1920.,
    //            height: 1080.,
    //        },
    //        ..OrthographicProjection::default_2d()
    //    }),
    //));
    //commands.spawn((
    //    Camera3d::default(),
    //    Camera {
    //        order: 1,
    //        ..default()
    //    },
    //    Transform::from_xyz(0., 0., 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    //    RenderLayers::layer(0),
    //));
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, sample: Res<Ambient>) {
    let bg = asset_server.load("private/cave-blue.png");
    commands.spawn((
        Sprite {
            image: bg,
            //image_mode: SpriteImageMode::Scale(ScalingMode::FillStart),
            custom_size: Some(Vec2::new(1920., 1080.)),
            ..default()
        },
        Transform::from_xyz(0., 0., 1.),
        BackgroundImg,
        MineSceneTag,
    ));

    let rock_layers: [Handle<Image>; 4] = (0..=3)
        .map(|i| asset_server.load(format!("private/rock-layer{}.png", i)))
        .collect::<Vec<_>>()
        .try_into()
        .expect("Expectd exactly 4 rock layers");

    let [rock1, rock2, rock3, rock4] = rock_layers;

    commands.spawn((
        Sprite::from_image(rock1),
        OriginalTransform(Transform::from_xyz(120., 50., 1.).with_scale(Vec3::splat(0.75))),
        Rock,
        Bouncer::default(),
        MineSceneTag,
    ));

    commands.spawn((
        Sprite::from_image(rock2),
        OriginalTransform(Transform::from_xyz(100., -50., 1.).with_scale(Vec3::splat(0.75))),
        Rock,
        Bouncer::default(),
        MineSceneTag,
    ));

    commands.spawn((
        Sprite::from_image(rock3),
        OriginalTransform(Transform::from_xyz(-150., 0., 1.1).with_scale(Vec3::splat(0.75))),
        Rock,
        Bouncer::default(),
        MineSceneTag,
    ));

    commands.spawn((
        Sprite {
            image: rock4,
            //anchor: bevy::sprite::Anchor::TopLeft,
            ..default()
        },
        OriginalTransform(
            Transform::from_xyz(-1920. / 2. + 100., 0., 1.).with_scale(Vec3::splat(0.745)),
        ),
        Rock,
        Bouncer::default(),
        MineSceneTag,
    ));

    commands.spawn((
        AudioPlayer::new(sample.0.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: Volume::Linear(0.75),
            ..default()
        },
        MineSceneTag,
    ));
}

fn load_audio(asset_server: Res<AssetServer>, mut commands: Commands) {
    let samples: Vec<Handle<AudioSource>> = (1..=6)
        .map(|i| asset_server.load(format!("private/non-commercial/punch/{}.ogg", i)))
        .collect();

    commands.insert_resource(Ambient(
        asset_server.load("private/non-commercial/ambient/music.ogg"),
    ));
    commands.insert_resource(MoneySpill(asset_server.load("private/money-spill-2.ogg")));
    commands.insert_resource(AudioSamples { samples });
}

fn load_gltf(
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
        RenderLayers::layer(1),
        MineSceneTag,
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
        RenderLayers::layer(1),
        MineSceneTag,
    ));

    //let obj: Handle<Mesh> = asset_server.load("private/coin.obj");

    let mesh_handle: Handle<Mesh> = asset_server.load(mesh_coin_path());
    commands.insert_resource(MyHandles {
        coin: mesh_handle.clone(),
    });

    let new_mat = StandardMaterial {
        base_color: Color::linear_rgb(1.0, 0.75, 0.1),
        perceptual_roughness: 0.25,
        metallic: 0.96,
        reflectance: 0.96,

        opaque_render_method: OpaqueRendererMethod::Auto,
        ..Default::default()
    };

    let mat = materials.add(new_mat);

    commands.insert_resource(MyMaterials { coin: mat.clone() });
}

fn spawn_coins(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &Res<MyMaterials>,
    at: Vec2,
) {
    let mesh: Handle<Mesh> = asset_server.get_handle(mesh_coin_path()).expect("No mesh");
    println!("{:?}", at);

    for x in 1..=6 {
        for y in 1..=6 {
            let r1 = thread_rng().gen_range(-1.0..1.0);
            let r2 = thread_rng().gen_range(-1.0..1.0);
            let r3 = thread_rng().gen_range(-1.0..1.0);
            commands.spawn((
                Coin {},
                Mesh3d(mesh.clone()),
                RigidBody::Dynamic,
                AngularVelocity(Vec3::new(r1 * 2., r2 * 2., r3 * 2.)),
                LinearVelocity(Vec3::new(r1 * 3., 2.5 + r2, 10. + r3)),
                Collider::sphere(0.1),
                MeshMaterial3d(materials.coin.clone()),
                Transform::from_translation((at / 20.).extend(-20. + x as f32 + y as f32))
                    .looking_at(Vec3::ZERO, Vec3::X),
                RenderLayers::layer(1),
                MineSceneTag,
            ));
        }
    }
}

fn mouse_button_input(
    buttons: Res<ButtonInput<MouseButton>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), (With<Camera2d>, With<RenderLayers>)>,
    //mut bouncer_q: Query<&mut Bouncer>,
    mut commands: Commands,
    q_rocks: Query<(&Transform, &Sprite, &mut Bouncer, &mut Health), With<Rock>>,
    images: Res<Assets<Image>>,
    samples: Res<AudioSamples>,
) {
    for (transform, sprite, mut bouncer, mut health) in q_rocks {
        if let Some(texture) = images.get(sprite.image.id()) {
            let texture_size = texture.size();
            let scale = transform.scale.truncate();
            let size = texture_size.as_vec2() * scale;
            //println!("{:?}", size);
            let half_size = size * 0.5;
            let pos = transform.translation.truncate();

            let min = pos - half_size;
            let max = pos + half_size;

            //println!("{:?}", size);

            if buttons.just_pressed(MouseButton::Left) {
                let window = q_window.single().expect("Window must exist");
                let (camera, camera_transform) = q_camera.single().expect("More than one camera");

                if let Some(cursor) = window.cursor_position() {
                    match camera.viewport_to_world(camera_transform, cursor) {
                        Ok(ray) => {
                            let world_position = ray.origin.truncate();
                            if (world_position.x >= min.x && world_position.x <= max.x)
                                && (world_position.y >= min.y && world_position.y <= max.y)
                            {
                                bouncer.bounce();
                                health.hit(34.);
                                let sample = samples.samples.choose(&mut thread_rng()).unwrap();
                                commands.spawn((
                                    AudioPlayer::new(sample.clone()),
                                    PlaybackSettings {
                                        mode: bevy::audio::PlaybackMode::Despawn,
                                        volume: Volume::Linear(1.0),
                                        ..default()
                                    },
                                    MineSceneTag,
                                ));
                                println!("Cursor over sprite at {:?}", transform.translation);
                                return;
                            }
                            println!("World coords: {}/{}", world_position.x, world_position.y);
                        }
                        Err(err) => {
                            eprintln!("Error converting cursor to world ray: {:?}", err);
                        }
                    }
                }
            }
        }
    }
}

fn update(
    time: Res<Time>,
    mut q_rocks: Query<(&mut Transform, &OriginalTransform, &mut Bouncer), With<Rock>>,
) {
    let delta = time.delta();

    for (mut transform, original_transform, mut bouncer) in q_rocks.iter_mut() {
        bouncer.update(delta);
        transform.scale = Vec3::splat(original_transform.scale.x + bouncer.pos / 10.0);
        transform.translation = original_transform.translation;
    }

    //println!("{}", bouncer.pos);
}

fn clean_dead(
    mut commands: Commands,
    q: Query<(Entity, &Health, &Transform)>,
    asset_server: Res<AssetServer>,
    materials: Res<MyMaterials>,
    money_spill: Res<MoneySpill>,
) {
    for (entity, hp, tr) in q.iter() {
        if hp.0 < 0. {
            spawn_coins(
                &mut commands,
                &asset_server,
                &materials,
                tr.translation.truncate(),
            );
            commands.spawn((
                AudioPlayer::new(money_spill.0.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: Volume::Linear(0.75),
                    ..default()
                },
                MineSceneTag,
            ));
            commands.entity(entity).despawn();
        }
    }
}

fn rotate_coin(time: Res<Time>, mut transform_q: Query<&mut Transform, With<Coin>>) {
    let delta = time.delta();

    let mut coin_t = transform_q.single_mut().unwrap();
    coin_t.rotate_y(delta.as_secs_f32() * 4.);
}
