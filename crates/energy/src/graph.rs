use std::collections::HashSet;

use bevy::prelude::*;
use bevy::utils::petgraph::prelude::*;
use bevy::utils::petgraph::visit::IntoNodeReferences;
use bevy::utils::Instant;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use de_core::gamestate::GameState;
use de_core::objects::Active;
use de_core::projection::ToFlat;
use de_index::SpatialQuery;
use de_spawner::{DespawnEventsPlugin, DespawnedComponentsEvent};
use parry3d::bounding_volume::Aabb;
use parry3d::math::Point;
use smallvec::{smallvec, SmallVec};

// The max distance (in meters) between two entities for them to be consider neighbors in the graph
const MAX_DISTANCE: f32 = 10.0;

pub(crate) struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DebugLinesPlugin::default(),
            DespawnEventsPlugin::<&NearbyUnits, NearbyUnits>::default(),
        ))
        .add_systems(OnEnter(GameState::Playing), setup)
        .add_systems(OnExit(GameState::Playing), clean_up)
        .add_systems(PostUpdate, spawn_graph_components)
        .add_systems(
            PreUpdate,
            (
                remove_old_nodes.before(GraphSystemSet::UpdateNearby),
                update_nearby_recv.in_set(GraphSystemSet::UpdateNearby),
                update_graph
                    .in_set(GraphSystemSet::UpdateGraph)
                    .after(GraphSystemSet::UpdateNearby),
                debug_lines.after(GraphSystemSet::UpdateGraph),
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
enum GraphSystemSet {
    UpdateNearby,
    UpdateGraph,
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

#[derive(Component, Default, Debug, Clone)]
pub struct NearbyUnits(SmallVec<[Nearby; 2]>, Option<Vec2>);

fn setup(mut commands: Commands) {
    commands.insert_resource(PowerGrid::default());
}

fn clean_up(mut commands: Commands) {
    commands.remove_resource::<PowerGrid>();
}

/// This system spawns Energy Producers and Energy Receivers and nearby units.
fn spawn_graph_components(
    mut commands: Commands,
    newly_spawned_units: Query<Entity, Added<Active>>,
) {
    for entity in newly_spawned_units.iter() {
        commands
            .entity(entity)
            .insert((EnergyReceiver, NearbyUnits::default()));
    }
}

fn update_nearby_recv(
    spacial_index_producer: SpatialQuery<Entity, With<EnergyProducer>>,
    spacial_index_receiver: SpatialQuery<Entity, With<EnergyReceiver>>,
    mut nearby_units: Query<(Entity, &mut NearbyUnits, &Transform), Changed<Transform>>,
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
                Point::from(transform.translation - Vec3::splat(MAX_DISTANCE)),
                Point::from(transform.translation + Vec3::splat(MAX_DISTANCE)),
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
    nearby_units: Query<(Entity, &NearbyUnits), Changed<NearbyUnits>>>,
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

fn remove_old_nodes(
    mut power_grid: ResMut<PowerGrid>,
    mut nearby_units: Query<&mut NearbyUnits>,
    mut death_events: EventReader<DespawnedComponentsEvent<NearbyUnits>>,
) {
    for event in death_events.iter() {
        power_grid.graph.remove_node(event.entity);

        // Remove the entity from the nearby units of all nearby units
        for outer_nearby in event
            .data
            .0
            .iter()
            .flat_map(|nearby| nearby.clone().into_inner())
        {
            for inner_nearby in nearby_units.get_mut(outer_nearby).unwrap().0.iter_mut() {
                match inner_nearby {
                    Nearby::Producer(producer) => producer.retain(|entity| *entity != event.entity),
                    Nearby::Receiver(receiver) => receiver.retain(|entity| *entity != event.entity),
                }
            }
        }
    }
}

fn debug_lines(
    power_grid: Res<PowerGrid>,
    query: Query<&Transform>,
    mut debug_lines: ResMut<DebugLines>,
) {
    for (node, _) in power_grid.graph.node_references() {
        let node_location = query.get(node).unwrap().translation;
        for neighbor in power_grid.graph.neighbors(node) {
            let neighbor_location = query.get(neighbor).unwrap().translation;
            debug_lines.line_colored(node_location, neighbor_location, 0., Color::RED);
        }
    }
}
