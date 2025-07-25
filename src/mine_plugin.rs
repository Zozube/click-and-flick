use crate::states::GameState;
use crate::util::despawn_screen;
use bevy_asset_loader::prelude::*;

use avian3d::prelude::*;
use bevy::asset::AssetPath;
use bevy::render::view::RenderLayers;
use bevy::{audio::Volume, pbr::OpaqueRendererMethod, prelude::*};
use bevy_simple_screen_boxing::CameraBox;
use core::time::Duration;
use rand::Rng;
use rand::{seq::SliceRandom, thread_rng};
use std::ops::{Deref, DerefMut};

#[derive(AssetCollection, Resource)]
struct SceneAssets {
    #[asset(path = "private/cave-blue.png")]
    background: Handle<Image>,

    #[asset(path = "private/non-commercial/ambient/music.ogg")]
    ambient: Handle<AudioSource>,

    #[asset(path = "private/money-spill-2.ogg")]
    money_spill: Handle<AudioSource>,

    #[asset(
        paths(
            "private/non-commercial/punch/1.ogg",
            "private/non-commercial/punch/2.ogg",
            "private/non-commercial/punch/3.ogg",
            "private/non-commercial/punch/4.ogg",
            "private/non-commercial/punch/5.ogg",
            "private/non-commercial/punch/6.ogg",
        ),
        collection(typed)
    )]
    hits: Vec<Handle<AudioSource>>,

    #[asset(
        paths(
            "private/rock-layer0.png",
            "private/rock-layer1.png",
            "private/rock-layer2.png",
            "private/rock-layer3.png",
        ),
        collection(typed)
    )]
    rocks: Vec<Handle<Image>>,
}

#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
enum MyLoadingStates {
    #[default]
    Inactive,
    Started,
    Ready,
}

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
        app.init_state::<MyLoadingStates>()
            .add_loading_state(
                LoadingState::new(MyLoadingStates::Started)
                    .continue_to_state(MyLoadingStates::Ready)
                    .load_collection::<SceneAssets>(),
            )
            .add_systems(
                OnEnter(GameState::Mine),
                |mut next_state: ResMut<NextState<MyLoadingStates>>| {
                    next_state.set(MyLoadingStates::Started)
                },
            )
            .add_systems(
                OnEnter(MyLoadingStates::Ready),
                (setup, setup_camera, load_gltf),
            )
            .add_systems(
                Update,
                (/*mouse_button_input, */ update, clean_dead)
                    .run_if(in_state(GameState::Mine).and(in_state(MyLoadingStates::Ready))),
            )
            .add_systems(OnExit(GameState::Mine), despawn_screen::<MineSceneTag>);
    }
}

fn setup_camera(mut commands: Commands) {
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
        MineSceneTag,
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
        MineSceneTag,
    ));
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, assets: Res<SceneAssets>) {
    commands.spawn((
        Sprite {
            image: assets.background.clone(),
            //image_mode: SpriteImageMode::Scale(ScalingMode::FillStart),
            custom_size: Some(Vec2::new(1920., 1080.)),
            ..default()
        },
        Name::new("Background"),
        Transform::from_xyz(0., 0., 1.),
        Pickable::default(),
        BackgroundImg,
        MineSceneTag,
    ));

    let [rock1, rock2, rock3, rock4]: [Handle<Image>; 4] = (0..4)
        .map(|i| assets.rocks.get(i).unwrap().clone())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    commands
        .spawn((
            Sprite::from_image(rock1),
            OriginalTransform(Transform::from_xyz(120., 50., 2.).with_scale(Vec3::splat(0.75))),
            Rock,
            Bouncer::default(),
            Pickable::default(),
            MineSceneTag,
        ))
        .observe(rock_click);

    commands
        .spawn((
            Sprite::from_image(rock2),
            OriginalTransform(Transform::from_xyz(100., -50., 3.).with_scale(Vec3::splat(0.75))),
            Rock,
            Bouncer::default(),
            Pickable::default(),
            MineSceneTag,
        ))
        .observe(rock_click);

    commands
        .spawn((
            Sprite::from_image(rock3),
            OriginalTransform(Transform::from_xyz(-150., 0., 4.).with_scale(Vec3::splat(0.75))),
            Rock,
            Bouncer::default(),
            Pickable::default(),
            MineSceneTag,
        ))
        .observe(rock_click);

    commands
        .spawn((
            Sprite {
                image: rock4,
                //anchor: bevy::sprite::Anchor::TopLeft,
                ..default()
            },
            OriginalTransform(
                Transform::from_xyz(-1920. / 2. + 100., 0., 5.).with_scale(Vec3::splat(0.745)),
            ),
            Rock,
            Bouncer::default(),
            Pickable::default(),
            MineSceneTag,
        ))
        .observe(rock_click);

    commands.spawn((
        AudioPlayer::new(assets.ambient.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: Volume::Linear(0.75),
            ..default()
        },
        MineSceneTag,
    ));
}

fn rock_click(
    trigger: Trigger<Pointer<Click>>,
    mut entities: Query<(&mut Bouncer, &mut Health)>,
    mut commands: Commands,
    assets: Res<SceneAssets>,
) {
    let entity = entities.get_mut(trigger.target());
    if entity.is_ok() {
        let entity = entity.unwrap();
        let (mut bouncer, mut health) = entity;
        bouncer.bounce();
        health.hit(34.);
        let sample = assets.hits.choose(&mut thread_rng()).unwrap();
        commands.spawn((
            AudioPlayer::new(sample.clone()),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: Volume::Linear(1.0),
                ..default()
            },
            MineSceneTag,
        ));
    }
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
    assets: Res<SceneAssets>,
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
                AudioPlayer::new(assets.money_spill.clone()),
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
