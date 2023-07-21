use bevy::prelude::*;
use de_core::{objects::ObjectType, player::Player, state::AppState};
use de_objects::Health;

use crate::{ObjectCounter, SpawnerSet};

pub(crate) struct DestroyerPlugin;

impl Plugin for DestroyerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            destroy
                .run_if(in_state(AppState::InGame))
                .in_set(SpawnerSet::Destroyer),
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
