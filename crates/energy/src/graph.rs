use std::collections::HashSet;

use bevy::prelude::*;
use bevy::utils::petgraph::prelude::*;
#[cfg(feature = "energy_graph_debug_lines")]
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use de_core::gamestate::GameState;
use de_core::projection::ToFlat;
use de_core::state::AppState;
use de_index::SpatialQuery;
use de_spawner::{DespawnEventsPlugin, DespawnedComponentsEvent};
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
        app.add_plugins(DespawnEventsPlugin::<&NearbyUnits, NearbyUnits>::default())
            .add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(OnExit(AppState::InGame), clean_up)
            .add_systems(
                PostUpdate,
                spawn_graph_components.run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                remove_old_nodes
                    .before(GraphSystemSet::UpdateNearby)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (
                    update_nearby.in_set(GraphSystemSet::UpdateNearby),
                    update_graph
                        .in_set(GraphSystemSet::UpdateGraph)
                        .after(GraphSystemSet::UpdateNearby),
                )
                    .run_if(in_state(AppState::InGame)),
            );

        #[cfg(feature = "energy_graph_debug_lines")]
        app.add_plugins(DebugLinesPlugin::default())
            .add_systems(PostUpdate, debug_lines.run_if(in_state(AppState::InGame)));
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
enum GraphSystemSet {
    UpdateNearby,
    UpdateGraph,
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

/// The power grid resource is used to store the power grid graph.
#[derive(Resource, Debug, Clone)]
pub(crate) struct PowerGrid {
    /// The power grid graph.
    graph: GraphMap<Entity, f64, Undirected>,
}

impl Default for PowerGrid {
    fn default() -> Self {
        Self {
            graph: GraphMap::new(),
        }
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
        self.units.retain(|e| *e != entity);
        println!("removed {:?} from {:?}", entity, self.units);
    }

    pub fn len(&self) -> usize {
        self.units.len()
    }

    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(PowerGrid::default());
}

fn clean_up(mut commands: Commands) {
    commands.remove_resource::<PowerGrid>();
}

/// This system spawns Energy Producers and Energy Receivers and nearby units.
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
    mut nearby_units: Query<(Entity, &mut NearbyUnits, &Transform), Changed<Transform>>,
) {
    nearby_units
        .par_iter_mut()
        .for_each_mut(|(entity, mut nearby_units, transform)| {
            let current_pos = transform.translation.to_flat();
            if let Some(last_pos) = nearby_units.last_pos {
                if current_pos.distance_squared(last_pos) < MIN_DISTANCE_FOR_UPDATE {
                    return;
                }
            }
            nearby_units.last_pos = Some(current_pos);

            let aabb = &Aabb::new(
                Point::from(transform.translation - Vec3::splat(MAX_DISTANCE)),
                Point::from(transform.translation + Vec3::splat(MAX_DISTANCE)),
            );

            let members = spacial_index_member.query_aabb(aabb, Some(entity));

            nearby_units.units.clear();

            nearby_units.units.extend(members.map(NearbyEntity));
        });
}

fn update_graph(
    mut power_grid: ResMut<PowerGrid>,
    nearby_units: Query<(Entity, &NearbyUnits), Changed<NearbyUnits>>,
) {
    for (entity, nearby_units) in nearby_units.iter() {
        let mut edges_to_remove = HashSet::new();

        edges_to_remove.extend(
            &mut power_grid
                .graph
                .edges(entity)
                .map(|edge| (edge.source(), edge.target())),
        );

        let mut edges_to_add = vec![];

        for nearby_entity in &nearby_units.units {
            edges_to_add.push((entity, nearby_entity.0));
        }

        for edge in edges_to_add.iter() {
            if !power_grid.graph.contains_edge(edge.0, edge.1) {
                power_grid.graph.add_edge(edge.0, edge.1, 1.0);
            }
            if edges_to_remove.contains(edge) {
                edges_to_remove.remove(edge);
            }
        }

        for edge in edges_to_remove.iter() {
            power_grid.graph.remove_edge(edge.0, edge.1);
        }
    }
}

fn remove_old_nodes(
    mut power_grid: ResMut<PowerGrid>,
    mut nearby_units_query: Query<&mut NearbyUnits>,
    mut death_events: EventReader<DespawnedComponentsEvent<NearbyUnits>>,
) {
    for event in death_events.iter() {
        power_grid.graph.remove_node(event.entity);

        for neighbor in power_grid.graph.neighbors(event.entity) {
            if let Ok(mut nearby_units) = nearby_units_query.get_mut(neighbor) {
                nearby_units.remove_matching(event.entity.into());
            }
        }
    }
}

#[cfg(feature = "energy_graph_debug_lines")]
fn debug_lines(
    power_grid: Res<PowerGrid>,
    query: Query<&Transform>,
    mut debug_lines: ResMut<DebugLines>,
) {
    for (from, to, _) in power_grid.graph.all_edges() {
        let from = query.get(from).unwrap();
        let to = query.get(to).unwrap();

        // if let (Ok(from), Ok(to)) = (from, to) {
        debug_lines.line_colored(from.translation, to.translation, 0., Color::RED);
        // }
    }
}
