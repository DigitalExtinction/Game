use bevy::prelude::*;
use de_core::{
    gamestate::GameState, gconfig::GameConfig, gresult::GameResult, stages::GameStage,
    state::AppState,
};
use iyes_loopless::prelude::*;

use crate::ObjectCounter;

pub(crate) struct GameEndPlugin;

impl Plugin for GameEndPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            GameStage::PostUpdate,
            game_end_detection_system.run_in_state(GameState::Playing),
        );
    }
}

fn game_end_detection_system(
    mut commands: Commands,
    conf: Res<GameConfig>,
    counter: Res<ObjectCounter>,
) {
    let mut result = None;
    if counter.player(conf.player()).unwrap().total() == 0 {
        result = Some(GameResult::new(false));
    } else if conf
        .players()
        .all(|player| conf.is_local_player(player) || counter.player(player).unwrap().total() == 0)
    {
        result = Some(GameResult::new(true));
    }

    if let Some(result) = result {
        commands.insert_resource(result);
        commands.insert_resource(NextState(AppState::InMenu));
    }
}
