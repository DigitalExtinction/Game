use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use de_core::{
    projection::{ToFlat, ToMsl},
    stages::GameStage,
    state::GameState,
};
use de_pathing::ScheduledPath;
use iyes_loopless::prelude::*;

use crate::{movement::Movement, MAX_ACCELERATION, MAX_SPEED};

const DESTINATION_ACCURACY: f32 = 0.1;

pub(crate) struct PathingPlugin;

impl Plugin for PathingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
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

fn follow_path(
    mut commands: Commands,
    mut objects: Query<(Entity, &Transform, &mut ScheduledPath, &mut Movement)>,
) {
    let finished = Arc::new(Mutex::new(Vec::new()));

    objects.par_for_each_mut(512, |(entity, transform, mut path, mut movement)| {
        let remaining = path.destination().distance(transform.translation.to_flat());
        if remaining <= DESTINATION_ACCURACY {
            finished.lock().unwrap().push(entity);
            movement.stop();
        } else {
            let advancement = path.advance(transform.translation.to_flat(), MAX_SPEED * 0.5);
            let direction = (advancement.to_msl() - transform.translation).normalize();
            let desired_speed = MAX_SPEED.min((2. * remaining * MAX_ACCELERATION).sqrt());
            movement.set_desired_velocity(desired_speed * direction);
        }
    });

    for entity in finished.lock().unwrap().drain(..) {
        commands.entity(entity).remove::<ScheduledPath>();
    }
}
