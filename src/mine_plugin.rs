use crate::states::GameState;

use bevy::render::Render;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy::{audio::Volume, pbr::OpaqueRendererMethod, prelude::*};
use core::time::Duration;
use rand::{seq::SliceRandom, thread_rng};

#[derive(Component)]
struct Coin {}

#[derive(Component)]
pub struct BackgroundImg;

#[derive(Component)]
pub struct Rock;

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
        Transform::from_xyz(0., 0., 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::layer(0),
    ));
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
        RenderLayers::layer(1),
        BackgroundImg,
    ));

    let rock_layers: [Handle<Image>; 4] = (0..=3)
        .map(|i| asset_server.load(format!("private/rock-layer{}.png", i)))
        .collect::<Vec<_>>()
        .try_into()
        .expect("Expectd exactly 4 rock layers");

    let [rock1, rock2, rock3, rock4] = rock_layers;

    commands.spawn((
        Sprite::from_image(rock1),
        Transform::from_xyz(120., 50., 1.).with_scale(Vec3::splat(0.75)),
        RenderLayers::layer(1),
        Rock,
        Bouncer::default(),
    ));

    commands.spawn((
        Sprite::from_image(rock2),
        Transform::from_xyz(100., -50., 1.).with_scale(Vec3::splat(0.75)),
        RenderLayers::layer(1),
        Rock,
        Bouncer::default(),
    ));

    commands.spawn((
        Sprite::from_image(rock3),
        Transform::from_xyz(-150., 0., 1.1).with_scale(Vec3::splat(0.75)),
        RenderLayers::layer(1),
        Rock,
        Bouncer::default(),
    ));

    commands.spawn((
        Sprite {
            image: rock4,
            anchor: bevy::sprite::Anchor::TopLeft,
            ..default()
        },
        Transform::from_xyz(-1920. / 2., 1080. / 2., 1.).with_scale(Vec3::splat(0.75)),
        RenderLayers::layer(1),
        Rock,
        Bouncer::default(),
    ));
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
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), (With<Camera2d>, With<RenderLayers>)>,
    //mut bouncer_q: Query<&mut Bouncer>,
    mut commands: Commands,
    q_rocks: Query<(&Transform, &Sprite, &mut Bouncer), With<Rock>>,
    images: Res<Assets<Image>>,
    samples: Res<AudioSamples>,
) {
    //let mut bouncer = bouncer_q.single_mut().unwrap();
    if buttons.just_pressed(MouseButton::Left) {
        let window = q_window.single().expect("Window must exist");
        let (camera, camera_transform) = q_camera.single().expect("More than one camera");

        if let Some(cursor) = window.cursor_position() {
            match camera.viewport_to_world(camera_transform, cursor) {
                Ok(ray) => {
                    let world_position = ray.origin.truncate();
                    for (transform, sprite, mut bouncer) in q_rocks {
                        let texture = images.get(sprite.image.id()).unwrap();
                        let texture_size = texture.size();
                        let scale = transform.scale.truncate();
                        let size = texture_size.as_vec2() * scale;
                        let half_size = size * 0.5 * scale;
                        let pos = transform.translation.truncate();

                        let min = pos - half_size;
                        let max = pos + half_size;

                        println!("{:?}", size);

                        if (world_position.x >= min.x && world_position.x <= max.x)
                            && (world_position.y >= min.y && world_position.y <= max.y)
                        {
                            bouncer.bounce();
                            println!("Cursor over sprite at {:?}", transform.translation);
                        }
                    }
                    println!("World coords: {}/{}", world_position.x, world_position.y);
                }
                Err(err) => {
                    eprintln!("Error converting cursor to world ray: {:?}", err);
                }
            }
        }

        //bouncer.bounce();
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

fn update(time: Res<Time>, mut q_rocks: Query<(&mut Transform, &mut Bouncer), With<Rock>>) {
    let delta = time.delta();

    for (mut transform, mut bouncer) in q_rocks.iter_mut() {
        bouncer.update(delta);
        transform.scale = Vec3::splat(1.0 + bouncer.pos / 10.0);
    }

    //println!("{}", bouncer.pos);
}

fn rotate_coin(time: Res<Time>, mut transform_q: Query<&mut Transform, With<Coin>>) {
    let delta = time.delta();

    let mut coin_t = transform_q.single_mut().unwrap();
    coin_t.rotate_y(delta.as_secs_f32() * 4.);
}
