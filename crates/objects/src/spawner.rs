use bevy::{ecs::system::EntityCommands, prelude::*};
use de_core::{
    events::ResendEventPlugin,
    gconfig::GameConfig,
    objects::{ActiveObjectType, InactiveObjectType, MovableSolid, Playable, StaticSolid},
    state::GameState,
};
use de_map::description::{ActiveObject, InactiveObject, Object, ObjectType};
use iyes_loopless::prelude::*;

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
    server: Res<AssetServer>,
    mut events: EventReader<SpawnEvent>,
) {
    for event in events.iter() {
        let object = &event.object;

        let transform = object.placement().to_transform();
        let global_transform = GlobalTransform::from(transform);
        let mut entity_commands = commands.spawn_bundle((global_transform, transform));

        match object.object_type() {
            ObjectType::Active(object) => {
                spawn_active(&game_config, &server, &mut entity_commands, object)
            }
            ObjectType::Inactive(object) => spawn_inactive(&server, &mut entity_commands, object),
        }
    }
}

fn spawn_active(
    game_config: &GameConfig,
    server: &AssetServer,
    commands: &mut EntityCommands,
    object: &ActiveObject,
) {
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

    let model_name = match object.object_type() {
        ActiveObjectType::Base => "base",
        ActiveObjectType::PowerHub => "powerhub",
        ActiveObjectType::Attacker => "attacker",
    };
    spawn_model(server, commands, model_name);
}

fn spawn_inactive(server: &AssetServer, commands: &mut EntityCommands, object: &InactiveObject) {
    info!("Spawning inactive object {}", object.object_type());

    commands.insert(StaticSolid);
    let model_name = match object.object_type() {
        InactiveObjectType::Tree => "tree",
    };
    spawn_model(server, commands, model_name);
}

fn spawn_model(server: &AssetServer, commands: &mut EntityCommands, model_name: &str) {
    commands.with_children(|parent| {
        parent.spawn_scene(server.load(&format!("{}.glb#Scene0", model_name)));
    });
}
