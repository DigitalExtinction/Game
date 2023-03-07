use bevy::prelude::States;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub enum AppState {
    #[default]
    AppLoading,
    InMenu,
    InGame,
}
