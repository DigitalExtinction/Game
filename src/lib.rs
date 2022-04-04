pub mod game;
pub mod math;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppStates {
    Menu,
    Game,
}
