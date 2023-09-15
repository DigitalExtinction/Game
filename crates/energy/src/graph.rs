use std::sync::Mutex;

use bevy::prelude::*;
use de_core::gamestate::GameState;
use de_core::projection::ToFlat;
use de_core::state::AppState;
use de_index::SpatialQuery;
use de_spawner::DespawnedComponentsEvent;
use parry3d::bounding_volume::Aabb;
use parry3d::math::Point;
use tinyvec::TinyVec;

use crate::Battery;

/// The max distance (in meters) between two entities for them to be consider neighbors in the graph
const MAX_DISTANCE: f32 = 10.0;
/// Minimum distance squared traveled by an object to update its nearby units.
const MIN_DISTANCE_FOR_UPDATE: f32 = 0.5;

pub(crate) struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            spawn_graph_components.run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            remove_old_nodes.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            FixedUpdate,
            (update_nearby.in_set(GraphSystemSet::UpdateNearby),)
                .run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
enum GraphSystemSet {
    UpdateNearby,
}

/// wrapped entity to allow for default values (so we can work with TinyVec)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NearbyEntity(Entity);

impl Default for NearbyEntity {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

impl From<NearbyEntity> for Entity {
    fn from(val: NearbyEntity) -> Self {
        val.0
    }
}

impl From<Entity> for NearbyEntity {
    fn from(entity: Entity) -> Self {
        Self(entity)
    }
}

/// The energy grid member component is used to store the energy grid member entities.
#[derive(Component, Debug, Clone)]
pub struct EnergyGridMember;

/// The nearby units component is used to store the nearby entities of an entity.
#[derive(Component, Default, Debug, Clone)]
pub struct NearbyUnits {
    units: TinyVec<[NearbyEntity; 16]>,
    last_pos: Option<Vec2>,
}

impl NearbyUnits {
    fn remove_matching(&mut self, entity: NearbyEntity) {
        let index = match self.units.iter().position(|e| *e == entity) {
            Some(index) => index,
            None => return,
        };

        self.units.swap_remove(index);
    }

    pub fn len(&self) -> usize {
        self.units.len()
    }

    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }
}

/// This system inserts newly spawned units into the energy grid.
fn spawn_graph_components(
    mut commands: Commands,
    newly_spawned_units: Query<Entity, Added<Battery>>,
) {
    for entity in newly_spawned_units.iter() {
        commands
            .entity(entity)
            .insert((EnergyGridMember, NearbyUnits::default()));
    }
}

pub fn update_nearby(
    spacial_index_member: SpatialQuery<Entity, With<EnergyGridMember>>,
    mut units: Query<(Entity, &mut NearbyUnits, &Transform), Changed<Transform>>,
) {
    let add_to: Mutex<Vec<(Entity, Entity)>> = Mutex::new(Vec::new());
    let remove_from: Mutex<Vec<(Entity, Entity)>> = Mutex::new(Vec::new());

    units
        .par_iter_mut()
        .for_each_mut(|(entity, mut nearby, transform)| {
            let current_pos = transform.translation.to_flat();
            if let Some(last_pos) = nearby.last_pos {
                if current_pos.distance_squared(last_pos) < MIN_DISTANCE_FOR_UPDATE {
                    return;
                }
            }
            nearby.last_pos = Some(current_pos);

            let aabb = &Aabb::new(
                Point::from(transform.translation - Vec3::splat(MAX_DISTANCE)),
                Point::from(transform.translation + Vec3::splat(MAX_DISTANCE)),
            );

            let original_units = nearby
                .units
                .drain(..)
                .collect::<TinyVec<[NearbyEntity; 16]>>();
            let new_nearby_units = spacial_index_member
                .query_aabb(aabb, Some(entity))
                .map(NearbyEntity)
                .collect::<TinyVec<[NearbyEntity; 16]>>();

            // get difference between original and new nearby units
            let mut to_add = Vec::new();
            let mut to_remove = Vec::new();

            for nearby_entity in &new_nearby_units {
                if !original_units.contains(nearby_entity) {
                    to_add.push(nearby_entity);
                }
            }

            for original_entity in &original_units {
                if !new_nearby_units.contains(original_entity) {
                    to_remove.push(original_entity);
                }
            }

            // by deferring the locking to here we dont waste too much time locking or keep it
            // locked for too long

            let mut add_to = add_to.lock().unwrap();
            for nearby_entity in to_add {
                add_to.push((nearby_entity.0, entity));
            }

            let mut remove_from = remove_from.lock().unwrap();
            for original_entity in to_remove {
                remove_from.push((original_entity.0, entity));
            }

            nearby.units = new_nearby_units;
        });

    let add_to = add_to.into_inner().unwrap();
    let remove_from = remove_from.into_inner().unwrap();

    for (entity, nearby_entity) in add_to {
        if let Ok((_, mut nearby, _)) = units.get_mut(entity) {
            nearby.units.push(nearby_entity.into());
        }
    }

    for (entity, nearby_entity) in remove_from {
        if let Ok((_, mut nearby, _)) = units.get_mut(entity) {
            nearby.remove_matching(nearby_entity.into());
        }
    }
}

fn remove_old_nodes(
    mut nearby_units_query: Query<&mut NearbyUnits>,
    mut death_events: EventReader<DespawnedComponentsEvent<NearbyUnits>>,
) {
    for event in death_events.iter() {
        let mut units_to_process = Vec::new();

        if let Ok(mut nearby_units) = nearby_units_query.get_mut(event.entity) {
            for unit in nearby_units.units.drain(..) {
                units_to_process.push(unit);
            }
        }

        for unit in units_to_process {
            if let Ok(mut nearby_units) = nearby_units_query.get_mut(unit.0) {
                nearby_units.remove_matching(NearbyEntity(event.entity));
            }
        }
    }
}
