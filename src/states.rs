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

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    Mine,
    Tavern,
    Map,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Mine
    }
}

// Very minimally implemented. Allow running systems separately and follow DIP
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameLogic;
