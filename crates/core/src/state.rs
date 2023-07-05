use bevy::prelude::{States, SystemSet};

use crate::transition::StateWithSet;

/// High level state of the application.
///
/// The application might enter each state multiple times. For example when the
/// user finishes a game and then starts a new one.
///
/// Some sub-systems (e.g. menu, game) control different finer-grained states
/// during each application state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub enum AppState {
    #[default]
    AppLoading,
    InMenu,
    /// A game has started.
    ///
    /// It may be loading, being played or finalizing.
    ///
    /// Before a game is started, make sure it is properly configured. Resource
    /// [`crate::gconfig::GameConfig`] must exist.
    InGame,
}

impl StateWithSet for AppState {
    type Set = AppStateSet;

    fn state_set() -> Self::Set {
        AppStateSet
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
pub struct AppStateSet;
