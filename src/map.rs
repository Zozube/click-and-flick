use crate::states::GameState;
use bevy::render::view::RenderLayers;
use bevy::{app::FixedMain, prelude::*};
use bevy_pancam::PanCam;
use bevy_simple_screen_boxing::CameraBox;
use std::cmp;
use std::collections::HashMap;

pub struct MapPlugin;

#[derive(Component)]
pub struct MapSceneTag;

#[derive(Resource)]
struct SceneAssets {
    map: Handle<Image>,
    mask: Handle<Image>,
}

#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
enum LoadingStates {
    #[default]
    Started,
    Ready,
}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LoadingStates>()
            .add_systems(OnEnter(LoadingStates::Started), load)
            .add_systems(
                FixedMain,
                poll_loading.run_if(in_state(LoadingStates::Started)),
            )
            .add_systems(OnEnter(LoadingStates::Ready), setup)
            .add_systems(Update, update);
    }
}

fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SceneAssets {
        map: asset_server.load("private/map.png"),
        mask: asset_server.load("private/mask.png"),
    });
}

fn poll_loading(
    asset_server: Res<AssetServer>,
    assets: Res<SceneAssets>,
    mut next_state: ResMut<NextState<LoadingStates>>,
) {
    let map_loaded = asset_server.is_loaded(assets.map.id());
    let mask_loaded = asset_server.is_loaded(assets.mask.id());

    if map_loaded && mask_loaded {
        next_state.set(LoadingStates::Ready);
    }
}

fn setup(
    mut commands: Commands,
    assets: Res<SceneAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        PanCam::default(),
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
        Sprite {
            image: assets.map.clone(),
            //image_mode: SpriteImageMode::Scale(ScalingMode::FillStart),
            //custom_size: Some(Vec2::new(1920., 1080.)),
            ..default()
        },
        Transform::from_xyz(0., 0., 1.),
        MapSceneTag,
    ));

    let (boxes, points) = match images.get_mut(&assets.mask) {
        Some(image) => process_map(image),
        None => (HashMap::new(), Vec::new()),
    };

    let mat = materials.add(Color::linear_rgba(0.5, 0.5, 0.33, 0.75));
    let mat2 = materials.add(Color::linear_rgba(0.75, 0.75, 0.75, 0.85));

    for point in points {
        println!("{:?}", point);
        let mesh = meshes.add(Rhombus::new(15., 15.));
        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(mat2.clone()),
            Transform::from_xyz(point.x, point.y, 21.),
        ));
    }

    for (color, rect) in boxes {
        println!("{:?}: {:?}", color, rect);
        let size = rect.max - rect.min;
        let mesh = meshes.add(Rectangle::from_size(size));
        let mid = rect.min.midpoint(rect.max);
        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(mat.clone()),
            Transform::from_xyz(mid.x, mid.y, 20.),
        ));
    }

    commands.spawn((
        Sprite {
            image: assets.mask.clone(),
            //image_mode: SpriteImageMode::Scale(ScalingMode::FillStart),
            //custom_size: Some(Vec2::new(1920., 1080.)),
            ..default()
        },
        Transform::from_xyz(0., 0., 10.),
        MapSceneTag,
    ));
}

fn process_map(image: &Image) -> (HashMap<String, Rect>, Vec<Vec2>) {
    let mut boxes: HashMap<String, URect> = HashMap::new();
    let mut points: Vec<Vec2> = Vec::new();

    let width = image.size().x as u32;
    let height = image.size().y as u32;

    let half_width = width / 2;
    let half_height = height / 2;

    // Instead of checkin all points, use grid of at least 100 lines
    let step = cmp::min(width, height) / 100;
    let x_steps = width / step;
    let y_steps = height / step;

    let flip_y = Vec2 { x: 1., y: -1. };

    let offset = Vec2 {
        x: half_width as f32,
        y: half_height as f32,
    };

    for x_step in 0..x_steps {
        for y_step in 0..y_steps {
            // Restore steps to pixels
            let x = x_step * step;
            let y = y_step * step;
            if let Ok(clr) = image.get_color_at(x, y) {
                if clr.alpha() == 1. {
                    let name = clr.hue().to_string();
                    let pos = UVec2 { x, y };

                    points.push((pos.as_vec2() - offset) * flip_y);

                    let rect = boxes.entry(name).or_insert(URect { min: pos, max: pos });

                    // Skip for fresh entries
                    if pos == rect.min && rect.min == rect.max {
                        continue;
                    }

                    if pos.x < rect.min.x {
                        rect.min.x = pos.x;
                    } else if pos.x > rect.max.x {
                        rect.max.x = pos.x;
                    }

                    if pos.y < rect.min.y {
                        rect.min.y = pos.y;
                    } else if pos.y > rect.max.y {
                        rect.max.y = pos.y;
                    }
                }
            }
        }
    }

    return (
        boxes
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    // Bevy sprite coordinates are relative to center by default
                    Rect {
                        min: (v.min.as_vec2() - offset) * flip_y,
                        max: (v.max.as_vec2() - offset) * flip_y,
                    },
                )
            })
            .collect(),
        points,
    );
}

fn post_load() {}

fn update() {}
