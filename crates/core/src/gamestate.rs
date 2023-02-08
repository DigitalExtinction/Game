use bevy::prelude::*;
use iyes_loopless::prelude::*;
use iyes_progress::prelude::*;

use crate::state::AppState;

pub(super) struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state_before_stage(CoreStage::PreUpdate, GameState::None)
            .add_plugin(ProgressPlugin::new(GameState::Loading).continue_to(GameState::Playing))
            .add_enter_system(AppState::InGame, setup)
            .add_exit_system(AppState::InGame, cleanup);
    }
}

/// Phase of an already started game. The game might be still loading or
/// finishing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    None,
    Loading,
    Playing,
}

fn setup(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::Loading));
}

fn cleanup(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::None));
}
