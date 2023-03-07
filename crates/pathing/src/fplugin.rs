use std::{ops::Deref, sync::Arc};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use de_core::{
    baseset::GameSet,
    gamestate::GameState,
    objects::{ObjectType, StaticSolid},
    state::AppState,
};
use de_map::size::MapBounds;
use de_objects::{IchnographyCache, ObjectCache};
use futures_lite::future;

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
        app.add_event::<PathFinderUpdated>()
            .add_system(setup_loading.in_schedule(OnEnter(AppState::InGame)))
            .add_system(setup_playing.in_schedule(OnEnter(GameState::Playing)))
            .add_system(cleanup.in_schedule(OnExit(AppState::InGame)))
            .add_system(
                check_removed
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(AppState::InGame))
                    .in_set(FinderSet::CheckRemoved),
            )
            .add_system(
                check_updated
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(GameState::Playing))
                    .in_set(FinderSet::CheckUpdated),
            )
            .add_system(
                update
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(GameState::Playing))
                    .after(FinderSet::CheckUpdated)
                    .after(FinderSet::CheckRemoved),
            )
            .add_system(
                check_update_result
                    .in_base_set(GameSet::PreMovement)
                    .run_if(in_state(GameState::Playing))
                    .in_set(FinderSet::UpdateFinder),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub(crate) enum FinderSet {
    UpdateFinder,
    CheckRemoved,
    CheckUpdated,
}

/// This event is sent whenever the path finder is updated.
///
/// Paths found before the event was sent may no longer be optimal or may lead
/// through non-accessible area.
pub(crate) struct PathFinderUpdated;

#[derive(Clone, Resource)]
pub(crate) struct FinderRes(Arc<PathFinder>);

impl FinderRes {
    fn new(finder: PathFinder) -> Self {
        Self(Arc::new(finder))
    }

    fn update(&mut self, finder: PathFinder) {
        self.0 = Arc::new(finder);
    }
}

impl Deref for FinderRes {
    type Target = PathFinder;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

#[derive(Resource)]
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

fn setup_loading(mut commands: Commands) {
    commands.init_resource::<UpdateFinderState>();
}

fn setup_playing(mut commands: Commands, bounds: Res<MapBounds>) {
    commands.insert_resource(FinderRes::new(PathFinder::new(bounds.as_ref())));
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<UpdateFinderState>();
    commands.remove_resource::<FinderRes>();
}

fn check_removed(
    mut state: ResMut<UpdateFinderState>,
    mut removed: RemovedComponents<StaticSolid>,
) {
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
    mut state: ResMut<UpdateFinderState>,
    mut finder_res: ResMut<FinderRes>,
    mut pf_updated: EventWriter<PathFinderUpdated>,
) {
    if let Some(finder) = state.check_result() {
        info!("Inserting updated path finder");
        finder_res.update(finder);
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
