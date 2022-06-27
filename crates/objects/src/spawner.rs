use bevy::prelude::*;
use de_core::{
    gconfig::GameConfig,
    objects::{ActiveObjectType, MovableSolid, ObjectType, Playable, StaticSolid},
    player::Player,
    state::GameState,
};
use iyes_loopless::prelude::*;

use crate::cache::ObjectCache;

pub(crate) struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn.run_in_state(GameState::Playing));
    }
}

#[derive(Bundle)]
pub struct SpawnBundle {
    object_type: ObjectType,
    transform: Transform,
    global_transform: GlobalTransform,
    spawn: Spawn,
}

impl SpawnBundle {
    pub fn new(object_type: ObjectType, transform: Transform) -> Self {
        Self {
            object_type,
            transform,
            global_transform: transform.into(),
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
    to_spawn: Query<(Entity, &ObjectType, Option<&Player>), With<Spawn>>,
) {
    for (entity, &object_type, player) in to_spawn.iter() {
        info!("Spawning object {}", object_type);

        let mut entity_commands = commands.entity(entity);
        entity_commands.remove::<Spawn>().with_children(|parent| {
            parent.spawn_scene(cache.get(object_type).scene());
        });

        match object_type {
            ObjectType::Active(active_type) => {
                let player = *player.expect("Active object without an associated was spawned.");
                if player == game_config.player() {
                    entity_commands.insert(Playable);
                }

                if active_type == ActiveObjectType::Attacker {
                    entity_commands.insert(MovableSolid);
                } else {
                    entity_commands.insert(StaticSolid);
                }
            }
            ObjectType::Inactive(_) => {
                entity_commands.insert(StaticSolid);
            }
        }
    }
}
