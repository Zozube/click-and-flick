#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::prelude::*;
use core::time::Duration;


#[derive(Resource)]
struct MyTimer(Timer);

#[derive(Component, Default)]
struct Bouncer {
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(MyTimer(Timer::from_seconds(1.0, TimerMode::Once)))
        .add_systems(Startup, (setup, setup_camera,))
        .add_systems(Update, (mouse_button_input, update))
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

fn mouse_button_input(
    buttons: Res<ButtonInput<MouseButton>>,
    mut timer: ResMut<MyTimer>,
    mut bouncer_q: Query<&mut Bouncer>,
) {
    let mut bouncer = bouncer_q.single_mut().unwrap();
    if buttons.just_pressed(MouseButton::Left) {
        println!("Mouse");
        timer.0.reset();
        bouncer.bounce();
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
    println!("{}", bouncer.pos);
}
