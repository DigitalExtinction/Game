use bevy::prelude::*;
use iyes_progress::ProgressPlugin;

use crate::{gconfig::GameConfig, nested_state, state::AppState};

pub(crate) struct GameStateSetupPlugin;

impl Plugin for GameStateSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(GameStatePlugin)
            .add_plugin(ProgressPlugin::new(GameState::Loading).continue_to(GameState::Playing));
    }
}

nested_state!(
    AppState::InGame -> GameState,
    doc = "Phase of an already started game. The game might be still loading or finishing.",
    enter = setup,
    exit = cleanup,
    variants = {
        Loading,
        Playing,
    }
);

fn setup(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Loading);
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<GameConfig>();
}
