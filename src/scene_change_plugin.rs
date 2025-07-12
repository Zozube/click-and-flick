use std::{f32::consts::PI, time::Duration};

use crate::states::{GameState, SceneTransitionState};
use bevy::prelude::*;

pub struct SceneChangePlugin;

#[derive(Component)]
pub struct SceneChangeOverlay;

#[derive(Component)]
pub struct InTransition {
    to: GameState,
    timer: Timer,
}

impl InTransition {
    pub fn new(to: GameState, duration: Duration) -> Self {
        Self {
            to,
            timer: Timer::new(duration, TimerMode::Once),
        }
    }
}

#[derive(Component)]
pub struct FadeIn {
    timer: Timer,
}

#[derive(Event)]
pub struct SceneChange {
    to: GameState,
}

impl FadeIn {
    pub fn new(duration: Duration) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Once),
        }
    }
}

impl Plugin for SceneChangePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SceneTransitionState>()
            .add_event::<SceneChange>()
            .add_systems(Startup, setup)
            .add_systems(Update, (handle_transitions, fade_in, handle_scene_chage))
            .add_systems(
                PreUpdate,
                (|state: Res<State<GameState>>, mut ev_scene_change: EventWriter<SceneChange>| {
                    let current_state = *state.get();
                    let new_state = match current_state {
                        GameState::Mine => GameState::Map,
                        GameState::Map => GameState::Tavern,
                        GameState::Tavern => GameState::Mine,
                    };
                    ev_scene_change.write(SceneChange { to: new_state });
                    println!("{:?}", new_state);
                })
                .distributive_run_if(
                    bevy::input::common_conditions::input_just_pressed(KeyCode::Tab),
                ),
            );
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::linear_rgba(0., 0., 0., 0.)),
        Pickable::IGNORE,
        SceneChangeOverlay,
    ));
}

fn fade_in(
    time: Res<Time>,
    mut commands: Commands,
    mut transitions: Query<(Entity, &mut FadeIn)>,
    mut overlay: Query<&mut BackgroundColor, With<SceneChangeOverlay>>,
) {
    for (entity, mut transition) in transitions.iter_mut() {
        transition.timer.tick(time.delta());
        let mut clr = overlay.single_mut().expect("not found");

        if transition.timer.finished() {
            commands.entity(entity).despawn();
            clr.0.set_alpha(0.0);
            continue;
        }

        let progress = transition.timer.elapsed_secs() / transition.timer.duration().as_secs_f32();
        let alpha = (progress * PI / 2.).cos();
        println!("{:?}", progress);
        clr.0.set_alpha(alpha);
    }
}

fn handle_scene_chage(mut commands: Commands, mut ev_scene_change: EventReader<SceneChange>) {
    for ev in ev_scene_change.read() {
        commands.spawn(InTransition::new(ev.to, Duration::from_secs_f32(0.2)));
    }
}

fn handle_transitions(
    time: Res<Time>,
    mut commands: Commands,
    mut transitions: Query<(Entity, &mut InTransition)>,
    mut overlay: Query<&mut BackgroundColor, With<SceneChangeOverlay>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (entity, mut transition) in transitions.iter_mut() {
        transition.timer.tick(time.delta());
        let mut clr = overlay.single_mut().expect("not found");

        if transition.timer.finished() {
            next_state.set(transition.to);
            commands.entity(entity).despawn();
            commands.spawn(FadeIn::new(Duration::from_secs_f32(0.2)));
            continue;
        }

        let progress = transition.timer.elapsed_secs() / transition.timer.duration().as_secs_f32();
        let alpha = (progress * PI / 2.).sin();
        clr.0.set_alpha(alpha);
    }
}
