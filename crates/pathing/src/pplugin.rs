use ahash::AHashMap;
use bevy::{
    ecs::query::Has,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use de_core::{
    gamestate::GameState,
    objects::MovableSolid,
    schedule::{PostMovement, PreMovement},
    state::AppState,
};
use de_types::{path::Path, projection::ToFlat};
use futures_lite::future;

use crate::{
    fplugin::{FinderRes, FinderSet, PathFinderUpdatedEvent},
    path::ScheduledPath,
    PathQueryProps, PathTarget,
};

const TARGET_TOLERANCE: f32 = 2.;

/// This plugin handles path finding requests and keeps scheduled paths
/// up-to-date.
pub struct PathingPlugin;

impl Plugin for PathingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateEntityPathEvent>()
            .add_event::<PathFoundEvent>()
            .add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(OnExit(AppState::InGame), cleanup)
            .add_systems(
                PreMovement,
                (
                    update_existing_paths
                        .run_if(on_event::<PathFinderUpdatedEvent>())
                        .in_set(PathingSet::UpdateExistingPaths)
                        .after(FinderSet::UpdateFinder),
                    update_requested_paths
                        .in_set(PathingSet::UpdateRequestedPaths)
                        .after(PathingSet::UpdateExistingPaths),
                    check_path_results
                        .in_set(PathingSet::PathResults)
                        // This system removes finished tasks from UpdatePathsState
                        // and inserts Scheduledpath components. When this happen,
                        // the tasks is no longer available however the component
                        // is not available as well until the end of the stage.
                        //
                        // System PathingSet::UpdateExistingPaths needs to detect
                        // that a path is either already scheduled or being
                        // computed. Thus this system must run after it.
                        .after(PathingSet::UpdateExistingPaths),
                    update_path_components
                        .after(PathingSet::PathResults)
                        // This is needed to avoid race condition in PathTarget
                        // removal which would happen if path was not-found before
                        // this system is run.
                        .before(PathingSet::UpdateRequestedPaths),
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostMovement,
                remove_path_targets.run_if(in_state(AppState::InGame)),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub(crate) enum PathingSet {
    UpdateRequestedPaths,
    UpdateExistingPaths,
    PathResults,
}

/// This event triggers computation of shortest path to a target and
/// replacement / insertion of this path to the entity.
#[derive(Event)]
pub struct UpdateEntityPathEvent {
    entity: Entity,
    target: PathTarget,
}

impl UpdateEntityPathEvent {
    /// # Arguments
    ///
    /// * `entity` - entity whose path should be updated / inserted. This must
    ///   be a locally simulated entity.
    ///
    /// * `target` - desired path target & path searching query configuration.
    pub fn new(entity: Entity, target: PathTarget) -> Self {
        Self { entity, target }
    }

    fn entity(&self) -> Entity {
        self.entity
    }

    fn target(&self) -> PathTarget {
        self.target
    }
}

/// This event is sent when a new path is found for a locally simulated entity.
#[derive(Event)]
pub(crate) struct PathFoundEvent {
    entity: Entity,
    path: Option<Path>,
}

impl PathFoundEvent {
    fn new(entity: Entity, path: Option<Path>) -> Self {
        Self { entity, path }
    }

    pub(crate) fn entity(&self) -> Entity {
        self.entity
    }

    pub(crate) fn path(&self) -> Option<&Path> {
        self.path.as_ref()
    }
}

#[derive(Default, Resource)]
struct UpdatePathsState {
    tasks: AHashMap<Entity, UpdatePathTask>,
}

impl UpdatePathsState {
    fn contains(&self, entity: Entity) -> bool {
        self.tasks.contains_key(&entity)
    }

    fn spawn_new(&mut self, finder: FinderRes, entity: Entity, source: Vec2, target: PathTarget) {
        let pool = AsyncComputeTaskPool::get();
        let task = pool.spawn(async move { finder.find_path(source, target) });
        self.tasks.insert(entity, UpdatePathTask::new(task));
    }

    fn check_results(&mut self) -> Vec<(Entity, Option<Path>)> {
        let mut results = Vec::new();
        self.tasks.retain(|&entity, task| match task.check() {
            UpdatePathState::Resolved(path) => {
                results.push((entity, path));
                false
            }
            UpdatePathState::Processing => true,
        });

        results
    }
}

struct UpdatePathTask(Task<Option<Path>>);

impl UpdatePathTask {
    fn new(task: Task<Option<Path>>) -> Self {
        Self(task)
    }

    fn check(&mut self) -> UpdatePathState {
        match future::block_on(future::poll_once(&mut self.0)) {
            Some(path) => UpdatePathState::Resolved(path),
            None => UpdatePathState::Processing,
        }
    }
}

enum UpdatePathState {
    Resolved(Option<Path>),
    Processing,
}

fn setup(mut commands: Commands) {
    commands.init_resource::<UpdatePathsState>()
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<UpdatePathsState>();
}

fn update_existing_paths(
    finder: Res<FinderRes>,
    mut state: ResMut<UpdatePathsState>,
    entities: Query<(Entity, &Transform, &PathTarget, Has<ScheduledPath>)>,
) {
    for (entity, transform, target, has_path) in entities.iter() {
        let position = transform.translation.to_flat();
        if !has_path && !state.contains(entity) {
            let current_distance = position.distance(target.location());
            let desired_distance = target.properties().distance();
            if (current_distance - desired_distance).abs() <= TARGET_TOLERANCE {
                continue;
            }
        }

        let new_target = PathTarget::new(
            target.location(),
            // Set max distance to infinity: the object has already departed
            // from the original point so it would not make sense to stop in
            // the middle of the path instead of getting as close as possible.
            PathQueryProps::new(target.properties().distance(), f32::INFINITY),
            target.permanent(),
        );

        state.spawn_new(finder.clone(), entity, position, new_target);
    }
}

fn update_requested_paths(
    mut commands: Commands,
    finder: Res<FinderRes>,
    mut state: ResMut<UpdatePathsState>,
    mut events: EventReader<UpdateEntityPathEvent>,
    entities: Query<&Transform, With<MovableSolid>>,
) {
    for event in events.iter() {
        if let Ok(transform) = entities.get(event.entity()) {
            commands.entity(event.entity()).insert(event.target());
            state.spawn_new(
                finder.clone(),
                event.entity(),
                transform.translation.to_flat(),
                event.target(),
            );
        }
    }
}

fn check_path_results(
    mut state: ResMut<UpdatePathsState>,
    mut events: EventWriter<PathFoundEvent>,
) {
    for (entity, path) in state.check_results() {
        events.send(PathFoundEvent::new(entity, path));
    }
}

fn update_path_components(
    mut commands: Commands,
    targets: Query<&PathTarget>,
    mut events: EventReader<PathFoundEvent>,
) {
    for event in events.iter() {
        let mut entity_commands = commands.entity(event.entity());
        match event.path() {
            Some(path) => {
                entity_commands.insert(ScheduledPath::new(path.clone()));
            }
            None => {
                entity_commands.remove::<ScheduledPath>();

                // This must be here on top of target removal in
                // remove_path_targets due to the possibility that
                // `ScheduledPath` was never found.
                if let Ok(target) = targets.get(event.entity()) {
                    if !target.permanent() {
                        entity_commands.remove::<PathTarget>();
                    }
                }
            }
        }
    }
}

fn remove_path_targets(
    mut commands: Commands,
    targets: Query<&PathTarget>,
    mut removed: RemovedComponents<ScheduledPath>,
) {
    for entity in removed.iter() {
        if let Ok(target) = targets.get(entity) {
            if !target.permanent() {
                commands.entity(entity).remove::<PathTarget>();
            }
        }
    }
}
