#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    InMenu,
    InGame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuState {
    None,
    MainMenu,
}

/// Phase of an already started game. The game might be still loading or
/// finishing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    None,
    Loading,
    Playing,
}
