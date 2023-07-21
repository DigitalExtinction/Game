use bevy::prelude::*;
use de_core::{
    schedule::{Movement, PreMovement},
    gamestate::GameState,
    projection::ToFlat,
    state::AppState,
};
use de_pathing::ScheduledPath;

use crate::{
    movement::{add_desired_velocity, DesiredVelocity},
    MAX_H_ACCELERATION, MAX_H_SPEED,
};

const DESTINATION_ACCURACY: f32 = 0.1;

pub(crate) struct PathingPlugin;

impl Plugin for PathingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreMovement,
            (
                finish_paths.run_if(in_state(GameState::Playing)),
                add_desired_velocity::<PathVelocity>.run_if(in_state(AppState::InGame)),
            ),
        )
        .add_systems(
            Movement,
            follow_path
                .run_if(in_state(GameState::Playing))
                .in_set(PathingSet::FollowPath),
        );
    }
}

pub(crate) struct PathVelocity;

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub(crate) enum PathingSet {
    FollowPath,
}

fn finish_paths(
    mut commands: Commands,
    mut objects: Query<(
        Entity,
        &Transform,
        &ScheduledPath,
        &mut DesiredVelocity<PathVelocity>,
    )>,
) {
    for (entity, transform, path, mut movement) in objects.iter_mut() {
        let remaining = path.destination().distance(transform.translation.to_flat());
        if remaining <= DESTINATION_ACCURACY {
            movement.stop();
            commands.entity(entity).remove::<ScheduledPath>();
        }
    }
}

fn follow_path(
    mut objects: Query<(
        &Transform,
        &mut ScheduledPath,
        &mut DesiredVelocity<PathVelocity>,
    )>,
) {
    objects
        .par_iter_mut()
        .for_each_mut(|(transform, mut path, mut movement)| {
            let location = transform.translation.to_flat();
            let remaining = path.destination().distance(location);
            let advancement = path.advance(location, MAX_H_SPEED * 0.5);
            let direction = (advancement - location).normalize();
            let desired_speed = MAX_H_SPEED.min((2. * remaining * MAX_H_ACCELERATION).sqrt());
            movement.update(desired_speed * direction);
        });
}
