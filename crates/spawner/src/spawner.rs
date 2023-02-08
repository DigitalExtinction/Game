#![allow(clippy::forget_non_drop)] // Needed because of #[derive(Bundle)]

use bevy::prelude::*;
use de_core::{
    gconfig::GameConfig,
    objects::{Active, ActiveObjectType, MovableSolid, ObjectType, Playable, StaticSolid},
    player::Player,
    stages::GameStage,
    state::GameState,
};
use de_objects::{IchnographyCache, InitialHealths, ObjectCache};
use de_terrain::CircleMarker;
use iyes_loopless::prelude::*;

use crate::ObjectCounter;

pub(crate) struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(GameStage::Update, spawn.run_in_state(GameState::Playing));
    }
}

#[derive(Bundle)]
pub struct SpawnBundle {
    object_type: ObjectType,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
    spawn: Spawn,
}

impl SpawnBundle {
    pub fn new(object_type: ObjectType, transform: Transform) -> Self {
        Self {
            object_type,
            transform,
            global_transform: transform.into(),
            visibility: Visibility::VISIBLE,
            computed_visibility: ComputedVisibility::INVISIBLE,
            spawn: Spawn,
        }
    }
}

#[derive(Component)]
struct Spawn;

fn spawn(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    cache: Res<ObjectCache>,
    healths: Res<InitialHealths>,
    mut counter: ResMut<ObjectCounter>,
    to_spawn: Query<(Entity, &ObjectType, Option<&Player>), With<Spawn>>,
) {
    for (entity, &object_type, player) in to_spawn.iter() {
        info!("Spawning object {}", object_type);

        let cache_item = cache.get(object_type);
        let mut entity_commands = commands.entity(entity);
        entity_commands.remove::<Spawn>().insert(cache_item.scene());

        match object_type {
            ObjectType::Active(active_type) => {
                entity_commands.insert(Active);

                let player = *player.expect("Active object without an associated was spawned.");
                counter.player_mut(player).unwrap().update(active_type, 1);

                if player == game_config.player() {
                    entity_commands.insert(Playable);
                }

                match active_type {
                    ActiveObjectType::Building(_) => {
                        entity_commands.insert(StaticSolid);
                    }
                    ActiveObjectType::Unit(_) => {
                        entity_commands.insert(MovableSolid);

                        let radius = cache.get_ichnography(object_type).radius();
                        entity_commands.insert(CircleMarker::new(radius));
                    }
                }

                entity_commands.insert(healths.health(active_type).clone());
                if let Some(cannon) = cache_item.cannon() {
                    entity_commands.insert(cannon.clone());
                }
            }
            ObjectType::Inactive(_) => {
                entity_commands.insert(StaticSolid);
            }
        }
    }
}
