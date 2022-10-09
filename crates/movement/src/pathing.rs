use bevy::prelude::*;
use de_core::{projection::ToFlat, stages::GameStage, state::GameState};
use de_pathing::ScheduledPath;
use iyes_loopless::prelude::*;

use crate::{movement::DesiredMovement, MAX_ACCELERATION, MAX_SPEED};

const DESTINATION_ACCURACY: f32 = 0.1;

pub(crate) struct PathingPlugin;

impl Plugin for PathingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            GameStage::PreMovement,
            finish_paths.run_in_state(GameState::Playing),
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

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum PathingLabels {
    FollowPath,
}

fn finish_paths(
    mut commands: Commands,
    mut objects: Query<(Entity, &Transform, &ScheduledPath, &mut DesiredMovement)>,
) {
    for (entity, transform, path, mut movement) in objects.iter_mut() {
        let remaining = path.destination().distance(transform.translation.to_flat());
        if remaining <= DESTINATION_ACCURACY {
            movement.set_velocity(Vec2::ZERO);
            commands.entity(entity).remove::<ScheduledPath>();
        }
    }
}

fn follow_path(mut objects: Query<(&Transform, &mut ScheduledPath, &mut DesiredMovement)>) {
    objects.par_for_each_mut(512, |(transform, mut path, mut movement)| {
        let location = transform.translation.to_flat();
        let remaining = path.destination().distance(location);
        let advancement = path.advance(location, MAX_SPEED * 0.5);
        let direction = (advancement - location).normalize();
        let desired_speed = MAX_SPEED.min((2. * remaining * MAX_ACCELERATION).sqrt());
        movement.set_velocity(desired_speed * direction);
    });
}
