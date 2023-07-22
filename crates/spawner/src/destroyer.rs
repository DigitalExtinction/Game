use bevy::prelude::*;
use de_audio::spatial::{PlaySpatialAudioEvent, Sound};
use de_core::{
    baseset::GameSet,
    objects::{ActiveObjectType, ObjectType},
    player::Player,
    state::AppState,
};
use de_objects::Health;

use crate::{ObjectCounter, SpawnerSet};

pub(crate) struct DestroyerPlugin;

impl Plugin for DestroyerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            destroy
                .in_base_set(GameSet::Update)
                .run_if(in_state(AppState::InGame))
                .in_set(SpawnerSet::Destroyer),
        );
    }
}

fn destroy(
    mut commands: Commands,
    mut counter: ResMut<ObjectCounter>,
    entities: Query<(Entity, &Player, &ObjectType, &Health, &Transform), Changed<Health>>,
    mut play_audio: EventWriter<PlaySpatialAudioEvent>,
) {
    for (entity, &player, &object_type, health, transform) in entities.iter() {
        if health.destroyed() {
            if let ObjectType::Active(active_type) = object_type {
                counter.player_mut(player).unwrap().update(active_type, -1);

                play_audio.send(PlaySpatialAudioEvent::new(
                    match active_type {
                        ActiveObjectType::Building(_) => Sound::DestroyBuilding,
                        ActiveObjectType::Unit(_) => Sound::DestroyUnit,
                    },
                    transform.translation,
                ));
            }

            commands.entity(entity).despawn_recursive();
        }
    }
}
