use bevy::prelude::*;
use de_core::{gamestate::GameState, gconfig::GameConfig, gresult::GameResult, state::AppState};

use crate::ObjectCounter;

pub(crate) struct GameEndPlugin;

impl Plugin for GameEndPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            game_end_detection_system.run_if(in_state(GameState::Playing)),
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

    let (playable, others) =
        counter
            .counters()
            .fold((0, 0), |(playable, others), (&player, counter)| {
                let total = counter.total();
                if conf.locals().is_playable(player) {
                    (playable + total, others)
                } else {
                    (playable, others + total)
                }
            });

    if playable == 0 {
        result = Some(GameResult::finished(false));
    } else if others == 0 {
        result = Some(GameResult::finished(true));
    }

    if let Some(result) = result {
        commands.insert_resource(result);
        next_state.set(AppState::InMenu);
    }
}
