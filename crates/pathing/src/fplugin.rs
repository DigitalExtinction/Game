use std::sync::Arc;

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use de_core::{
    objects::{ObjectType, StaticSolid},
    stages::GameStage,
    state::GameState,
};
use de_map::size::MapBounds;
use de_objects::{IchnographyCache, ObjectCache};
use futures_lite::future;
use iyes_loopless::prelude::*;

use crate::{exclusion::ExclusionArea, finder::PathFinder, triangulation::triangulate};

/// This plugin registers systems which automatically update the path finder
/// when static solid objects are added or removed from the world.
///
/// # World Update
///
/// * Each solid static object's ichnography (a convex polygon) is offset by
///   some amount. See [`crate::exclusion`].
///
/// * Overlapping polygons from the previous steps are merged -- their convex
///   hull is used. These are called exclusion areas.
///
/// * Whole map (surface) is triangulated with Constrained Delaunay
///   triangulation (CDT). All edges from the exclusion areas are used as
///   constrains. See [`crate::triangulation`].
///
/// * Triangles from inside the exclusion areas are dropped, remaining
///   triangles are used in successive steps.
///
/// * A visibility sub-graph is created. The each triangle edge is connected
///   with all neighboring triangle edges. See
///   [`crate::finder::PathFinder::from_triangles`].
pub struct FinderPlugin;

impl Plugin for FinderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UpdateFinderState>()
            .add_event::<PathFinderUpdated>()
            .add_enter_system(GameState::Playing, setup)
            .add_system_to_stage(
                GameStage::PostUpdate,
                check_removed
                    .run_in_state(GameState::Playing)
                    .label(FinderLabel::CheckRemoved),
            )
            .add_system_to_stage(
                GameStage::PostUpdate,
                check_updated
                    .run_in_state(GameState::Playing)
                    .label(FinderLabel::CheckUpdated),
            )
            .add_system_to_stage(
                GameStage::PostUpdate,
                update
                    .run_in_state(GameState::Playing)
                    .after(FinderLabel::CheckUpdated)
                    .after(FinderLabel::CheckRemoved),
            )
            .add_system_to_stage(
                GameStage::PreMovement,
                check_update_result
                    .run_in_state(GameState::Playing)
                    .label(FinderLabel::UpdateFinder),
            );
    }
}

#[derive(SystemLabel)]
pub(crate) enum FinderLabel {
    UpdateFinder,
    CheckRemoved,
    CheckUpdated,
}

/// This event is sent whenever the path finder is updated.
///
/// Paths found before the event was sent may no longer be optimal or may lead
/// through non-accessible area.
pub(crate) struct PathFinderUpdated;

struct UpdateFinderState {
    invalid: bool,
    task: Option<Task<PathFinder>>,
}

impl UpdateFinderState {
    fn invalidate(&mut self) {
        self.invalid = true;
    }

    fn should_update(&self) -> bool {
        self.invalid && self.task.is_none()
    }

    fn spawn_update<'a, T>(&mut self, cache: ObjectCache, bounds: MapBounds, entities: T)
    where
        T: Iterator<Item = (&'a Transform, &'a ObjectType)>,
    {
        debug_assert!(self.task.is_none());
        let entities: Vec<(Transform, ObjectType)> = entities
            .map(|(transform, object_type)| (*transform, *object_type))
            .collect();
        let pool = AsyncComputeTaskPool::get();
        self.task = Some(pool.spawn(async move { create_finder(cache, bounds, entities) }));
        self.invalid = false;
    }

    fn check_result(&mut self) -> Option<PathFinder> {
        let finder = self
            .task
            .as_mut()
            .and_then(|task| future::block_on(future::poll_once(task)));
        if finder.is_some() {
            self.task = None;
        }
        finder
    }
}

impl Default for UpdateFinderState {
    fn default() -> Self {
        Self {
            invalid: true,
            task: None,
        }
    }
}

type ChangedQuery<'world, 'state> =
    Query<'world, 'state, Entity, (With<StaticSolid>, Changed<Transform>)>;

fn setup(mut commands: Commands, bounds: Res<MapBounds>) {
    commands.insert_resource(Arc::new(PathFinder::new(bounds.as_ref())));
}

fn check_removed(mut state: ResMut<UpdateFinderState>, removed: RemovedComponents<StaticSolid>) {
    if removed.iter().next().is_some() {
        state.invalidate();
    }
}

fn check_updated(mut state: ResMut<UpdateFinderState>, changed: ChangedQuery) {
    if changed.iter().next().is_some() {
        state.invalidate();
    }
}

fn update(
    mut state: ResMut<UpdateFinderState>,
    bounds: Res<MapBounds>,
    cache: Res<ObjectCache>,
    entities: Query<(&Transform, &ObjectType), With<StaticSolid>>,
) {
    if state.should_update() {
        info!("Spawning path finder update task");
        state.spawn_update(cache.clone(), *bounds, entities.iter());
    }
}

fn check_update_result(
    mut commands: Commands,
    mut state: ResMut<UpdateFinderState>,
    mut pf_updated: EventWriter<PathFinderUpdated>,
) {
    if let Some(finder) = state.check_result() {
        info!("Inserting updated path finder");
        commands.insert_resource(Arc::new(finder));
        pf_updated.send(PathFinderUpdated);
    }
}

/// Creates a new path finder by triangulating accessible area on the map.
// This function has to be public due to its benchmark.
pub fn create_finder(
    cache: impl IchnographyCache,
    bounds: MapBounds,
    entities: Vec<(Transform, ObjectType)>,
) -> PathFinder {
    debug!(
        "Going to create a new path finder from {} entities",
        entities.len()
    );
    let exclusions = ExclusionArea::build(cache, entities.as_slice());
    let triangles = triangulate(&bounds, exclusions.as_slice());
    PathFinder::from_triangles(triangles, exclusions)
}
