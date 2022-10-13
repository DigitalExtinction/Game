use std::sync::Arc;

use ahash::AHashMap;
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use de_core::{objects::MovableSolid, projection::ToFlat, stages::GameStage, state::GameState};
use futures_lite::future;
use iyes_loopless::prelude::*;

use crate::{
    finder::PathFinder,
    fplugin::{FinderLabel, PathFinderUpdated},
    path::{Path, ScheduledPath},
    PathQueryProps, PathTarget,
};

const TARGET_TOLERANCE: f32 = 2.;

pub struct PathingPlugin;

impl Plugin for PathingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UpdatePathsState>()
            .add_event::<UpdateEntityPath>()
            .add_system_to_stage(
                GameStage::PreMovement,
                update_existing_paths
                    .run_in_state(GameState::Playing)
                    .label(PathingLabel::UpdateExistingPaths)
                    .after(FinderLabel::UpdateFinder),
            )
            .add_system_to_stage(
                GameStage::PreMovement,
                update_requested_paths
                    .run_in_state(GameState::Playing)
                    .after(PathingLabel::UpdateExistingPaths),
            )
            .add_system_to_stage(
                GameStage::PreMovement,
                check_path_results.run_in_state(GameState::Playing),
            )
            .add_system_to_stage(GameStage::PostUpdate, remove_path_targets);
    }
}

#[derive(SystemLabel)]
enum PathingLabel {
    UpdateExistingPaths,
}

/// This event triggers computation of shortest path to a target and
/// replacement / insertion of this path to the entity.
pub struct UpdateEntityPath {
    entity: Entity,
    target: PathTarget,
}

impl UpdateEntityPath {
    /// # Arguments
    ///
    /// * `entity` - entity whose path should be updated / inserted.
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

struct UpdatePathsState {
    tasks: AHashMap<Entity, UpdatePathTask>,
}

impl UpdatePathsState {
    fn spawn_new(
        &mut self,
        finder: Arc<PathFinder>,
        entity: Entity,
        source: Vec2,
        target: PathTarget,
    ) {
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

impl Default for UpdatePathsState {
    fn default() -> Self {
        Self {
            tasks: AHashMap::new(),
        }
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

fn update_existing_paths(
    finder: Res<Arc<PathFinder>>,
    mut state: ResMut<UpdatePathsState>,
    mut events: EventReader<PathFinderUpdated>,
    entities: Query<(Entity, &Transform, &PathTarget, Option<&ScheduledPath>)>,
) {
    if events.iter().count() == 0 {
        // consume the iterator
        return;
    }

    for (entity, transform, target, path) in entities.iter() {
        let position = transform.translation.to_flat();
        if path.is_none()
            && position.distance(target.location())
                <= (target.properties().distance() + TARGET_TOLERANCE)
        {
            continue;
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
    finder: Res<Arc<PathFinder>>,
    mut state: ResMut<UpdatePathsState>,
    mut events: EventReader<UpdateEntityPath>,
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

fn check_path_results(mut commands: Commands, mut state: ResMut<UpdatePathsState>) {
    for (entity, path) in state.check_results() {
        let mut entity_commands = commands.entity(entity);
        match path {
            Some(path) => {
                entity_commands.insert(ScheduledPath::new(path));
            }
            None => {
                entity_commands.remove::<ScheduledPath>();
            }
        }
    }
}

fn remove_path_targets(
    mut commands: Commands,
    entities: Query<(Entity, &PathTarget), Without<ScheduledPath>>,
) {
    for (entity, target) in entities.iter() {
        if !target.permanent() {
            commands.entity(entity).remove::<PathTarget>();
        }
    }
}
