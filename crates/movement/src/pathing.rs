use bevy::prelude::*;
use de_core::{gamestate::GameState, projection::ToFlat, stages::GameStage, state::AppState};
use de_pathing::ScheduledPath;
use iyes_loopless::prelude::*;

use crate::{
    movement::{add_desired_velocity, DesiredVelocity},
    MAX_H_ACCELERATION, MAX_H_SPEED,
};

const DESTINATION_ACCURACY: f32 = 0.1;

pub(crate) struct PathingPlugin;

impl Plugin for PathingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            GameStage::PreMovement,
            SystemSet::new()
                .with_system(finish_paths.run_in_state(GameState::Playing))
                .with_system(add_desired_velocity::<PathVelocity>.run_in_state(AppState::InGame)),
        )
        .add_system_set_to_stage(
            GameStage::Movement,
            SystemSet::new().with_system(
                follow_path
                    .run_in_state(GameState::Playing)
                    .label(PathingLabels::FollowPath),
            ),
        );
    }
}

pub(crate) struct PathVelocity;

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum PathingLabels {
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
    objects.par_for_each_mut(512, |(transform, mut path, mut movement)| {
        let location = transform.translation.to_flat();
        let remaining = path.destination().distance(location);
        let advancement = path.advance(location, MAX_H_SPEED * 0.5);
        let direction = (advancement - location).normalize();
        let desired_speed = MAX_H_SPEED.min((2. * remaining * MAX_H_ACCELERATION).sqrt());
        movement.update(desired_speed * direction);
    });
}
