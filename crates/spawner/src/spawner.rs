#![allow(clippy::forget_non_drop)] // Needed because of #[derive(Bundle)]

use bevy::prelude::*;
use de_core::{
    gconfig::GameConfig,
    objects::{ActiveObjectType, MovableSolid, ObjectType, Playable, StaticSolid},
    player::Player,
    stages::GameStage,
    state::GameState,
};
use de_objects::{InitialHealths, ObjectCache};
use iyes_loopless::prelude::*;

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
            visibility: Visibility::visible(),
            computed_visibility: ComputedVisibility::not_visible(),
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
    to_spawn: Query<(Entity, &ObjectType, Option<&Player>), With<Spawn>>,
) {
    for (entity, &object_type, player) in to_spawn.iter() {
        info!("Spawning object {}", object_type);

        let cache_item = cache.get(object_type);
        let mut entity_commands = commands.entity(entity);
        entity_commands.remove::<Spawn>().insert(cache_item.scene());

        match object_type {
            ObjectType::Active(active_type) => {
                let player = *player.expect("Active object without an associated was spawned.");
                if player == game_config.player() {
                    entity_commands.insert(Playable);
                }

                match active_type {
                    ActiveObjectType::Building(_) => {
                        entity_commands.insert(StaticSolid);
                    }
                    ActiveObjectType::Unit(_) => {
                        entity_commands.insert(MovableSolid);
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
