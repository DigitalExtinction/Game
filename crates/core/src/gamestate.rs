use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::state::AppState;

pub(super) struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_system(
                // TODO find a nicer way, i.e. without adding it twice
                apply_state_transition::<GameState>
                    .in_base_set(CoreSet::StateTransitions)
                    .after(apply_state_transition::<AppState>),
            )
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

fn setup(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Loading);
}

fn cleanup(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::None);
}
