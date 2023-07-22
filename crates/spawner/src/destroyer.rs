use bevy::{asset::LoadState, prelude::*};
use bevy_kira_audio::AudioSource as KAudioSource;
use de_audio::spatial::{SpatialSoundBundle, Volume};
use de_core::{
    baseset::GameSet,
    objects::{ActiveObjectType, ObjectType},
    player::Player,
    state::AppState,
};
use de_objects::Health;
use iyes_progress::{Progress, ProgressSystem};

use crate::{ObjectCounter, SpawnerSet};

pub(crate) struct DestroyerPlugin;

impl Plugin for DestroyerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(AppState::AppLoading)))
            .add_system(load.track_progress().run_if(in_state(AppState::AppLoading)))
            .add_system(
                destroy
                    .in_base_set(GameSet::Update)
                    .run_if(in_state(AppState::InGame))
                    .in_set(SpawnerSet::Destroyer),
            );
    }
}

#[derive(Resource)]
struct DestructionSounds {
    building: Handle<KAudioSource>,
    unit: Handle<KAudioSource>,
}

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(DestructionSounds {
        building: server.load("audio/sounds/destruction_building.ogg"),
        unit: server.load("audio/sounds/destruction_unit.ogg"),
    });
}

fn load(server: Res<AssetServer>, sounds: Res<DestructionSounds>) -> Progress {
    Progress {
        done: [&sounds.building, &sounds.unit]
            .into_iter()
            .map(|state| match server.get_load_state(state) {
                LoadState::Loaded => 1,
                LoadState::NotLoaded | LoadState::Loading => 0,
                _ => panic!("Unexpected loading state."),
            })
            .sum(),
        total: 2,
    }
}

fn destroy(
    mut commands: Commands,
    mut counter: ResMut<ObjectCounter>,
    entities: Query<(Entity, &Player, &ObjectType, &Health, &Transform), Changed<Health>>,
    sounds: Res<DestructionSounds>,
) {
    for (entity, &player, &object_type, health, transform) in entities.iter() {
        if health.destroyed() {
            if let ObjectType::Active(active_type) = object_type {
                counter.player_mut(player).unwrap().update(active_type, -1);

                commands.spawn((
                    TransformBundle::from_transform(*transform),
                    match active_type {
                        ActiveObjectType::Building(_) => SpatialSoundBundle {
                            sound: sounds.building.clone(),
                            ..Default::default()
                        },
                        ActiveObjectType::Unit(_) => SpatialSoundBundle {
                            sound: sounds.unit.clone(),
                            ..Default::default()
                        },
                    },
                ));
            }

            commands.entity(entity).despawn_recursive();
        }
    }
}
