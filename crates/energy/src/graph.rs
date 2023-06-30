use std::collections::HashSet;
use std::ops::Add;

use bevy::ecs::query::QueryParIter;
use bevy::prelude::*;
use bevy::utils::petgraph::prelude::*;
use bevy::utils::petgraph::visit::IntoNodeReferences;
use bevy::utils::{HashMap, Instant};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use de_core::baseset::GameSet;
use de_core::gamestate::GameState;
use de_core::objects::{MovableSolid, StaticSolid};
use de_core::projection::ToFlat;
use de_index::SpatialQuery;
use parry3d::bounding_volume::Aabb;
use parry3d::math::Point;
use smallvec::{smallvec, SmallVec};

// The max distance (in meters) between two entities for them to be consider neighbors in the graph
const MAX_DISTANCE: f32 = 10.0;

pub(crate) struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(DebugLinesPlugin::default())
            .add_system(setup.in_schedule(OnEnter(GameState::Playing)))
            .add_system(
                update_nearby_recv
                    .in_base_set(GameSet::PreUpdate)
                    .in_set(GraphSystemSet::UpdateNearby)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_system(
                update_graph
                    .in_base_set(GameSet::PreUpdate)
                    .in_set(GraphSystemSet::UpdateGraph)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_system(clean_up.in_schedule(OnExit(GameState::Playing)))
            .add_system(
                debug_lines
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
enum GraphSystemSet {
    UpdateNearby,
    UpdateGraph,
    CleanUp,
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

/// The energy receiver component is used to mark an entity as an energy receiver.
#[derive(Component, Debug, Clone, Copy)]
pub struct EnergyReceiver;

/// The energy producer component is used to mark an entity as an energy producer.
#[derive(Component, Debug, Clone, Copy)]
pub struct EnergyProducer;

/// The nearby component is used to store the nearby entities of an entity.
#[derive(Debug, Clone)]
pub enum Nearby {
    Receiver(SmallVec<[Entity; 40]>),
    Producer(SmallVec<[Entity; 40]>),
}

impl Nearby {
    fn into_inner(self) -> SmallVec<[Entity; 40]> {
        match self {
            Nearby::Receiver(inner) => inner,
            Nearby::Producer(inner) => inner,
        }
    }
}

#[derive(Component, Default)]
pub struct NearbyUnits(SmallVec<[Nearby; 2]>, Option<Vec2>);


fn setup(mut commands: Commands) {
    commands.insert_resource(PowerGrid::default());
}

fn clean_up(mut commands: Commands) {
    commands.remove_resource::<PowerGrid>();
}

fn update_nearby_recv(
    spacial_index_producer: SpatialQuery<Entity, With<EnergyProducer>>,
    spacial_index_receiver: SpatialQuery<Entity, With<EnergyReceiver>>,
    mut nearby_units: Query<(Entity, &mut NearbyUnits, &Transform)>,
) {
    let time = Instant::now();
    nearby_units
        .par_iter_mut()
        .for_each_mut(|(entity, mut nearby_units, transform)| {
            if let Some(last_pos) = nearby_units.1 {
                if transform.translation.to_flat().distance_squared(last_pos) < 0.5 {
                    return;
                }
            }
            nearby_units.1 = Some(transform.translation.to_flat());

            let aabb = &Aabb::new(
                Point::from(*(transform.translation - Vec3::splat(MAX_DISTANCE)).as_ref()),
                Point::from(*(transform.translation + Vec3::splat(MAX_DISTANCE)).as_ref()),
            );

            let producers = spacial_index_producer.query_aabb(aabb, Some(entity));

            let receivers = spacial_index_receiver.query_aabb(aabb, Some(entity));

            update_nearby(nearby_units, producers.collect(), receivers.collect());
        });
    println!("update_nearby_recv: {:?}", time.elapsed());
}

fn update_nearby(
    mut nearby_units: Mut<NearbyUnits>,
    producers: Vec<Entity>,
    receivers: Vec<Entity>,
) {
    let mut nearby_producers = SmallVec::new();
    let mut nearby_receivers = SmallVec::new();

    for producer in producers {
        nearby_producers.push(producer);
    }

    for receiver in receivers {
        nearby_receivers.push(receiver);
    }

    nearby_units.0 = smallvec![
        Nearby::Producer(nearby_producers),
        Nearby::Receiver(nearby_receivers)
    ];
}

fn update_graph(
    mut power_grid: ResMut<PowerGrid>,
    nearby_units: Query<(Entity, &NearbyUnits), Or<(Added<NearbyUnits>, Changed<NearbyUnits>)>>,
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
        for group in &nearby_units.0 {
            match group {
                Nearby::Producer(producers) => {
                    for producer in producers {
                        edges_to_add.push((entity, *producer));
                    }
                }
                Nearby::Receiver(receivers) => {
                    for receiver in receivers {
                        edges_to_add.push((entity, *receiver));
                    }
                }
                _ => {}
            }
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

// fn remove_old_nodes(
//     mut power_grid: ResMut<PowerGrid>,
//     mut nearby_removed: RemovedComponents<NearbyUnits>,
//     mut death_events: EventReader<DespawnedComponents<NearbyUnits>>,
// ) {
//     // for entity in nearby_removed.iter() {
//     //     // for edge in power_grid.graph.edges(entity).collect::<Vec<_>>() {
//     //     //     nearby_units.get_mut(edge.target()).unwrap().0.retain(|e| match e {
//     //     //         Nearby::Receiver(entity) => {}
//     //     //         Nearby::Producer(entity) => {}
//     //     //     } != entity);
//     //     // }
//     //
//     //     power_grid.graph.remove_node(entity);
//     // }
// }

fn debug_lines(
    power_grid: Res<PowerGrid>,
    query: Query<&Transform>,
    mut debug_lines: ResMut<DebugLines>,
) {
    let mut i = 0;
    for (node, _) in power_grid.graph.node_references() {
        let node_location = query.get(node).unwrap().translation;
        for neighbor in power_grid.graph.neighbors(node) {
            let neighbor_location = query.get(neighbor).unwrap().translation;
            debug_lines.line_colored(node_location, neighbor_location, 0., Color::RED);
            i += 1;
        }
    }
}
