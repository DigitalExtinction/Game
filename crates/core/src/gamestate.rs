use bevy::prelude::*;
use iyes_progress::ProgressPlugin;

use crate::{gconfig::GameConfig, nested_state, state::AppState};

pub(crate) struct GameStateSetupPlugin;

impl Plugin for GameStateSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GameStatePlugin,
            ProgressPlugin::new(GameState::Loading).continue_to(GameState::Waiting),
        ));
    }
}

nested_state!(
    AppState::InGame -> GameState,
    doc = "Phase of an already started game. The game might be still loading or finishing.",
    enter = setup,
    exit = cleanup,
    variants = {
        // The game is ready for initialization. Waiting for other players to
        // get to the state as well.
        Prepared,
        // The game is being initialized locally and possibly by other players
        // as well.
        Loading,
        // The game is locally initialized, waiting for other player to finish
        // initialization as well.
        Waiting,
        Playing,
    }
);

fn setup(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Prepared);
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<GameConfig>();
}
