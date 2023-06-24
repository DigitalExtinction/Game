use bevy::prelude::*;
use de_core::{baseset::GameSet, gamestate::GameState, projection::ToFlat, state::AppState};
use de_energy::Battery;
use de_pathing::ScheduledPath;

use crate::{
    movement::{add_desired_velocity, DesiredVelocity},
    MAX_H_ACCELERATION, MAX_H_SPEED,
};

const DESTINATION_ACCURACY: f32 = 0.1;

pub(crate) struct PathingPlugin;

impl Plugin for PathingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            finish_paths
                .in_base_set(GameSet::PreMovement)
                .run_if(in_state(GameState::Playing)),
        )
        .add_system(
            check_battery
                .in_base_set(GameSet::PreMovement)
                .in_set(PathingSet::CheckBattery)
                .run_if(in_state(GameState::Playing)),
        )
        .add_system(
            add_desired_velocity::<PathVelocity>
                .in_base_set(GameSet::PreMovement)
                .run_if(in_state(AppState::InGame)),
        )
        .add_system(
            follow_path
                .in_base_set(GameSet::Movement)
                .run_if(in_state(GameState::Playing))
                .after(PathingSet::CheckBattery)
                .in_set(PathingSet::FollowPath),
        );
    }
}

pub(crate) struct PathVelocity;

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub(crate) enum PathingSet {
    CheckBattery,
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

fn check_battery(mut objects: Query<(&mut DesiredVelocity<PathVelocity>, &Battery)>) {
    for (mut movement, battery) in objects.iter_mut() {
        if battery.energy() <= 0. {
            movement.pause();
        } else if movement.paused() {
            movement.resume();
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
            if movement.paused() {
                return;
            }
            let location = transform.translation.to_flat();
            let remaining = path.destination().distance(location);
            let advancement = path.advance(location, MAX_H_SPEED * 0.5);
            let direction = (advancement - location).normalize();
            let desired_speed = MAX_H_SPEED.min((2. * remaining * MAX_H_ACCELERATION).sqrt());
            movement.update(desired_speed * direction);
        });
}
