#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    InMenu,
    InGame,
}

/// Phase of an already started game. The game might be still loading or
/// finishing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    None,
    Loading,
    Playing,
}
