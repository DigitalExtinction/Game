use super::Labels;
use bevy::prelude::*;
use glam::Vec2;
use std::f32::consts::{FRAC_2_PI, PI};

const TARGET_ACCURACY: f32 = 0.1;
const MAX_ANGULAR_SPEED: f32 = PI;
const MAX_SPEED: f32 = 10.;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SendEntityEvent>().add_system_set(
            SystemSet::new()
                .with_system(process_events.after(Labels::InputUpdate))
                .with_system(move_objects),
        );
    }
}

pub struct SendEntityEvent {
    entity: Entity,
    target: Vec2,
}

impl SendEntityEvent {
    pub fn new(entity: Entity, target: Vec2) -> Self {
        Self { entity, target }
    }
}

#[derive(Component)]
struct Target {
    position: Vec2,
}

impl From<Vec2> for Target {
    fn from(position: Vec2) -> Self {
        Self { position }
    }
}

fn process_events(mut commands: Commands, mut events: EventReader<SendEntityEvent>) {
    for event in events.iter() {
        commands
            .entity(event.entity)
            .insert(Target::from(event.target));
    }
}

fn move_objects(
    mut commands: Commands,
    mut objects: Query<(Entity, &Target, &mut Transform)>,
    time: Res<Time>,
) {
    let time_delta = time.delta().as_secs_f32();

    for (entity, target, mut transform) in objects.iter_mut() {
        let target_3d = Vec3::new(target.position.x, 0., target.position.y);
        let object_to_target = target_3d - transform.translation;

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

        if (transform.translation - target_3d).length() < TARGET_ACCURACY {
            commands.entity(entity).remove::<Target>();
        }
    }
}
