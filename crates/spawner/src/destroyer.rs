use bevy::prelude::*;
use de_core::{stages::GameStage, state::GameState};
use de_objects::Health;
use iyes_loopless::prelude::*;

use crate::SpawnerLabels;

pub(crate) struct DestroyerPlugin;

impl Plugin for DestroyerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            GameStage::Update,
            destroy
                .run_in_state(GameState::Playing)
                .label(SpawnerLabels::Destroyer),
        );
    }
}

fn destroy(mut commands: Commands, entities: Query<(Entity, &Health)>) {
    for (entity, health) in entities.iter() {
        if health.destroyed() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
