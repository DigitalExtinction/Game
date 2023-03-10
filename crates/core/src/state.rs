use bevy::prelude::{States, SystemSet};

use crate::transition::StateWithSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub enum AppState {
    #[default]
    AppLoading,
    InMenu,
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
