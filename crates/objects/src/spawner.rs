use bevy::{ecs::system::EntityCommands, prelude::*};
use de_core::{
    events::ResendEventPlugin,
    gconfig::GameConfig,
    objects::{ActiveObjectType, MovableSolid, ObjectType, Playable, StaticSolid},
    state::GameState,
};
use de_map::description::{ActiveObject, InnerObject, Object};
use iyes_loopless::prelude::*;

use crate::cache::Cache;

pub(crate) struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEvent>()
            .add_plugin(ResendEventPlugin::<SpawnEvent>::default())
            .add_system(spawn.run_in_state(GameState::Playing));
    }
}

pub struct SpawnEvent {
    object: Object,
}

impl SpawnEvent {
    pub fn new(object: Object) -> Self {
        Self { object }
    }
}

fn spawn(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    cache: Res<Cache>,
    mut events: EventReader<SpawnEvent>,
) {
    for event in events.iter() {
        let object = &event.object;

        let transform = object.placement().to_transform();
        let global_transform = GlobalTransform::from(transform);
        let mut entity_commands = commands.spawn_bundle((global_transform, transform));

        let object_type = match object.inner() {
            InnerObject::Active(object) => {
                spawn_active(game_config.as_ref(), &mut entity_commands, object);
                ObjectType::Active(object.object_type())
            }
            InnerObject::Inactive(object) => {
                info!("Spawning inactive object {}", object.object_type());
                entity_commands.insert(StaticSolid);
                ObjectType::Inactive(object.object_type())
            }
        };

        entity_commands.with_children(|parent| {
            parent.spawn_scene(cache.get(object_type).scene());
        });
    }
}

fn spawn_active(game_config: &GameConfig, commands: &mut EntityCommands, object: &ActiveObject) {
    info!("Spawning active object {}", object.object_type());

    commands.insert(object.player());
    if object.player() == game_config.player() {
        commands.insert(Playable);
    }

    if object.object_type() == ActiveObjectType::Attacker {
        commands.insert(MovableSolid);
    } else {
        commands.insert(StaticSolid);
    }
}
