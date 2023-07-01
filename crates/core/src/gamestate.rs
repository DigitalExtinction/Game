use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    gconfig::GameConfig,
    state::AppState,
    transition::{DeStateTransition, StateWithSet},
};

pub(super) struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state_with_set::<GameState>()
            .configure_sets((AppState::state_set(), GameState::state_set()).chain())
            .add_plugin(ProgressPlugin::new(GameState::Loading).continue_to(GameState::Playing))
            .add_system(setup.in_schedule(OnEnter(AppState::InGame)))
            .add_system(cleanup.in_schedule(OnExit(AppState::InGame)));
    }
}

/// Phase of an already started game. The game might be still loading or
/// finishing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub enum GameState {
    #[default]
    None,
    Loading,
    Playing,
}

impl StateWithSet for GameState {
    type Set = GameStateSet;

    fn state_set() -> Self::Set {
        GameStateSet
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
pub struct GameStateSet;

fn setup(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Loading);
}

fn cleanup(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    commands.remove_resource::<GameConfig>();
    next_state.set(GameState::None);
}
