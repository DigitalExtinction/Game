use bevy::prelude::*;
use de_core::{objects::ObjectType, player::Player, stages::GameStage, state::AppState};
use de_objects::Health;
use iyes_loopless::prelude::*;

use crate::{ObjectCounter, SpawnerLabels};

pub(crate) struct DestroyerPlugin;

impl Plugin for DestroyerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            GameStage::Update,
            destroy
                .run_in_state(AppState::InGame)
                .label(SpawnerLabels::Destroyer),
        );
    }
}

fn destroy(
    mut commands: Commands,
    mut counter: ResMut<ObjectCounter>,
    entities: Query<(Entity, &Player, &ObjectType, &Health), Changed<Health>>,
) {
    for (entity, &player, &object_type, health) in entities.iter() {
        if health.destroyed() {
            if let ObjectType::Active(active_type) = object_type {
                counter.player_mut(player).unwrap().update(active_type, -1);
            }
            commands.entity(entity).despawn_recursive();
        }
    }
}
