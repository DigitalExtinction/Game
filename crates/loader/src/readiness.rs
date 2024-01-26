use bevy::prelude::*;
use de_core::{gamestate::GameState, gconfig::GameConfig};
use de_messages::Readiness;
use de_multiplayer::{GameReadinessEvent, SetReadinessEvent};

pub(crate) struct ReadinessPlugin;

impl Plugin for ReadinessPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Prepared),
            set_readiness(Readiness::Prepared),
        )
        .add_systems(
            OnEnter(GameState::Waiting),
            set_readiness(Readiness::Initialized),
        )
        .add_systems(
            Update,
            (
                progress(Readiness::Prepared, GameState::Loading)
                    .run_if(in_state(GameState::Prepared)),
                progress(Readiness::Initialized, GameState::Playing)
                    .run_if(in_state(GameState::Waiting)),
            ),
        );
    }
}

fn set_readiness(readiness: Readiness) -> impl Fn(EventWriter<SetReadinessEvent>) {
    move |mut events: EventWriter<SetReadinessEvent>| {
        events.send(SetReadinessEvent::from(readiness));
    }
}

fn progress(
    readiness: Readiness,
    target_state: GameState,
) -> impl Fn(Res<GameConfig>, EventReader<GameReadinessEvent>, ResMut<NextState<GameState>>) {
    move |conf: Res<GameConfig>,
          mut events: EventReader<GameReadinessEvent>,
          mut state: ResMut<NextState<GameState>>| {
        if !conf.multiplayer() || events.read().any(|e| **e == readiness) {
            state.set(target_state);
        }
    }
}
