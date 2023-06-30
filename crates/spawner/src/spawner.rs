#![allow(clippy::forget_non_drop)] // Needed because of #[derive(Bundle)]

use bevy::prelude::*;
use de_core::{
    gamestate::GameState,
    gconfig::GameConfig,
    objects::{Active, ActiveObjectType, MovableSolid, ObjectType, Playable, StaticSolid},
    player::Player,
};
use de_energy::{Battery, EnergyReceiver, NearbyUnits};
use de_objects::{AssetCollection, InitialHealths, SceneType, Scenes, SolidObjects};
use de_terrain::{CircleMarker, MarkerVisibility, RectangleMarker};
use smallvec::smallvec;

use crate::ObjectCounter;

pub(crate) struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn.run_if(in_state(GameState::Playing)));
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
    to_spawn: Query<(Entity, &ObjectType, &GlobalTransform, Option<&Player>), With<Spawn>>,
) {
    for (entity, &object_type, transform, player) in to_spawn.iter() {
        info!("Spawning object {}", object_type);

        let mut entity_commands = commands.entity(entity);
        entity_commands
            .remove::<Spawn>()
            .insert(scenes.get(SceneType::Solid(object_type)).clone());

        let solid = solids.get(object_type);
        match object_type {
            ObjectType::Active(active_type) => {
                entity_commands.insert(Active);
                entity_commands.insert(Battery::default());
                entity_commands.insert(EnergyReceiver);
                entity_commands.insert(NearbyUnits::default());

                let player = *player.expect("Active object without an associated was spawned.");
                counter.player_mut(player).unwrap().update(active_type, 1);

                if game_config.locals().is_playable(player) || cfg!(feature = "godmode") {
                    entity_commands.insert(Playable);
                }

                match active_type {
                    ActiveObjectType::Building(_) => {
                        entity_commands.insert(StaticSolid);

                        let local_aabb = solid.ichnography().local_aabb();
                        entity_commands
                            .insert(RectangleMarker::from_aabb_transform(local_aabb, transform));
                    }
                    ActiveObjectType::Unit(_) => {
                        entity_commands.insert(MovableSolid);

                        let radius = solid.ichnography().radius();
                        entity_commands.insert(CircleMarker::new(radius));
                    }
                }

                entity_commands.insert(MarkerVisibility::default());

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
