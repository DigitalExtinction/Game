use super::{
    config::GameConfig,
    objects::{Active, ActiveObjectType, Movable, Playable, SolidObject},
    positions::{MovingEntitiesTree, MovingTreeItem},
    GameStates,
};
use bevy::{
    hierarchy::BuildChildren,
    prelude::{
        App, AssetServer, Commands, EventReader, GlobalTransform, ParallelSystemDescriptorCoercion,
        Plugin, Res, ResMut, SpawnSceneAsChildCommands, SystemLabel, SystemSet, Transform,
    },
};
use glam::{Quat, Vec2, Vec3};

pub struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ToBeSpawnedEvent>().add_system_set(
            SystemSet::on_update(GameStates::Playing)
                .with_system(spawn.label(SpawnerLabels::Spawn)),
        );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub enum SpawnerLabels {
    Spawn,
}

pub struct ToBeSpawnedEvent {
    position: Vec2,
    rotation: f32,
    object_type: ActiveObjectType,
    player: u8,
}

impl ToBeSpawnedEvent {
    pub fn new(position: Vec2, rotation: f32, object_type: ActiveObjectType, player: u8) -> Self {
        Self {
            position,
            rotation,
            object_type,
            player,
        }
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn transform(&self) -> Transform {
        let translation = Vec3::new(self.position.x, 0., self.position.y);
        let rotation = Quat::from_rotation_y(self.rotation);
        Transform {
            translation,
            rotation,
            ..Default::default()
        }
    }
}

fn spawn(
    mut commands: Commands,
    mut tree: ResMut<MovingEntitiesTree>,
    server: Res<AssetServer>,
    game_config: Res<GameConfig>,
    mut to_be_spawned: EventReader<ToBeSpawnedEvent>,
) {
    for spawn_event in to_be_spawned.iter() {
        let transform = spawn_event.transform();
        let model_name = match spawn_event.object_type {
            ActiveObjectType::Base => "base",
            ActiveObjectType::PowerHub => "powerhub",
            ActiveObjectType::Attacker => "attacker",
        };

        let mut entity_commands = commands.spawn_bundle((
            GlobalTransform::from(transform),
            transform,
            SolidObject,
            Active::new(spawn_event.player),
            spawn_event.object_type,
        ));

        if spawn_event.player == game_config.player() {
            entity_commands.insert(Playable);
        }
        if spawn_event.object_type == ActiveObjectType::Attacker {
            entity_commands.insert(Movable);
            let moving_tree_item = tree.insert(entity_commands.id(), spawn_event.position());
            entity_commands.insert(moving_tree_item);
        }

        entity_commands.with_children(|parent| {
            parent.spawn_scene(server.load(&format!("{}.glb#Scene0", model_name)));
        });
    }
}
