#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    AppLoading,
    InMenu,
    InGame,
}
