use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    SplashScreen,
    LoadingScreen,
    MainMenu,
    InGame,
}

impl Default for AppState {
    fn default() -> Self {
        AppState::MainMenu
    }
}

#[derive(States, Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    Mine,
    Tavern,
    Map,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
/**
For animation of fade in / fade out between scene changes
*/
pub enum SceneTransitionState {
    #[default]
    Normal,
    FadeOut,
    Black,
    FadeIn,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Mine
    }
}

// Very minimally implemented. Allow running systems separately and follow DIP
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameLogic;
