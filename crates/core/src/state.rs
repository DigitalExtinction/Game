#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    InMenu,
    InGame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuState {
    None,
    // It is necessary to avoid name collision with `GameState::Loading`. This
    // is because Debug fmt is used by iyes_progress for a stage labeling.
    //
    // Alternatively, custom Debug implementation could have been be provided.
    MLoading,
    MainMenu,
    MapSelection,
    SignIn,
    GameListing,
}

/// Phase of an already started game. The game might be still loading or
/// finishing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    None,
    Loading,
    Playing,
}
