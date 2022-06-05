use std::f32::consts::{FRAC_2_PI, PI};

use bevy::prelude::*;
use de_core::{projection::ToMsl, state::GameState};
use de_pathing::Path;
use iyes_loopless::prelude::*;

const TARGET_ACCURACY: f32 = 0.1;
const MAX_ANGULAR_SPEED: f32 = PI;
const MAX_SPEED: f32 = 10.;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Update, update.run_in_state(GameState::Playing));
    }
}

fn update(
    mut commands: Commands,
    mut objects: Query<(Entity, &mut Path, &mut Transform)>,
    time: Res<Time>,
) {
    let time_delta = time.delta().as_secs_f32();

    for (entity, mut path, mut transform) in objects.iter_mut() {
        let object_to_target = loop {
            let target_3d = path.waypoints().last().unwrap().to_msl();
            let object_to_target = target_3d - transform.translation;

            if object_to_target.length() < TARGET_ACCURACY {
                if path.advance() {
                    commands.entity(entity).remove::<Path>();
                    break object_to_target;
                }
            } else {
                break object_to_target;
            }
        };

        let forward = transform.forward();
        let angle = forward.angle_between(object_to_target);

        if angle > f32::EPSILON {
            let direction = if forward.cross(object_to_target).y.is_sign_negative() {
                -1.
            } else {
                1.
            };
            let angle_delta = direction * (MAX_ANGULAR_SPEED * time_delta).min(angle);
            transform.rotate(Quat::from_rotation_y(angle_delta));
        }

        if angle >= FRAC_2_PI {
            continue;
        }

        let delta_scalar = MAX_SPEED * time_delta;
        let delta_vec = (delta_scalar * forward).clamp_length_max(object_to_target.dot(forward));
        transform.translation += delta_vec;
    }
}
