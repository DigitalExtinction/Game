#![allow(clippy::forget_non_drop)] // Needed because of #[derive(Bundle)]

use bevy::prelude::*;
use de_core::{
    baseset::GameSet,
    gamestate::GameState,
    gconfig::GameConfig,
    objects::{Active, ActiveObjectType, MovableSolid, ObjectType, Playable, StaticSolid},
    player::Player,
};
use de_energy::Battery;
use de_energy::EnergyUnit::{Megajoules};
use de_objects::{AssetCollection, InitialHealths, SceneType, Scenes, SolidObjects};
use de_terrain::CircleMarker;

use crate::ObjectCounter;

pub(crate) struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            spawn
                .in_base_set(GameSet::Update)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Bundle)]
pub struct SpawnBundle {
    object_type: ObjectType,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
    spawn: Spawn, }

impl SpawnBundle {
    pub fn new(object_type: ObjectType, transform: Transform) -> Self {
        Self {
            object_type,
            transform,
            global_transform: transform.into(),
            visibility: Visibility::Inherited,
            computed_visibility: ComputedVisibility::HIDDEN,
            spawn: Spawn,
        }
    }
}

#[derive(Component)]
struct Spawn;

fn spawn(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    scenes: Res<Scenes>,
    solids: SolidObjects,
    healths: Res<InitialHealths>,
    mut counter: ResMut<ObjectCounter>,
    to_spawn: Query<(Entity, &ObjectType, Option<&Player>), With<Spawn>>,
) {
    for (entity, &object_type, player) in to_spawn.iter() {
        info!("Spawning object {}", object_type);

        let mut entity_commands = commands.entity(entity);
        entity_commands
            .remove::<Spawn>()
            .insert(scenes.get(SceneType::Solid(object_type)).clone());

        let solid = solids.get(object_type);
        match object_type {
            ObjectType::Active(active_type) => {
                entity_commands.insert(Active);
                entity_commands.insert(Battery::default()); // a bit of a placeholder as final solution is likely going to be in `SolidObject` configured though the object config

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

                        let radius = solid.ichnography().radius();
                        entity_commands.insert(CircleMarker::new(radius));
                    }
                }

                entity_commands.insert(healths.health(active_type).clone());
                if let Some(cannon) = solid.cannon() {
                    entity_commands.insert(cannon.clone());
                }
            }
            ObjectType::Inactive(_) => {
                entity_commands.insert(StaticSolid);
            }
        }
    }
}
