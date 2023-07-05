use bevy::prelude::*;
use de_core::{
    baseset::GameSet, gamestate::GameState, gconfig::GameConfig, gresult::GameResult,
    state::AppState,
};

use crate::ObjectCounter;

pub(crate) struct GameEndPlugin;

impl Plugin for GameEndPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            game_end_detection_system
                .in_base_set(GameSet::PostUpdate)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn game_end_detection_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    conf: Res<GameConfig>,
    counter: Res<ObjectCounter>,
) {
    let mut result = None;
    if counter.player(conf.locals().playable()).unwrap().total() == 0 {
        result = Some(GameResult::new(false));
    } else if conf.players().all(|player| {
        conf.locals().is_playable(player) || counter.player(player).unwrap().total() == 0
    }) {
        result = Some(GameResult::new(true));
    }

    if let Some(result) = result {
        commands.insert_resource(result);
        next_state.set(AppState::InMenu);
    }
}
