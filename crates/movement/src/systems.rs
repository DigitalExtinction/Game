use std::{
    f32::consts::{FRAC_PI_4, PI, TAU},
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use de_core::{
    objects::MovableSolid,
    projection::{ToFlat, ToMsl},
    stages::GameStage,
    state::GameState,
};
use de_pathing::ScheduledPath;
use iyes_loopless::prelude::*;

const DESTINATION_ACCURACY: f32 = 0.1;
/// Maximum object speed in meters per second.
const MAX_SPEED: f32 = 10.;
/// Maximum object acceleration in meters per second squared.
const MAX_ACCELERATION: f32 = 2. * MAX_SPEED;
/// Maximum object angular velocity in radians per second.
const MAX_ANGULAR_SPEED: f32 = PI;

pub(crate) struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(GameStage::PreMovement, setup_entities)
            .add_system_set_to_stage(
                GameStage::Movement,
                SystemSet::new()
                    .with_system(
                        update_desired_velocity
                            .run_in_state(GameState::Playing)
                            .label(MovementLabels::UpdateDesiredVelocity),
                    )
                    .with_system(
                        kinematics
                            .run_in_state(GameState::Playing)
                            .label(MovementLabels::Kinematics)
                            .after(MovementLabels::UpdateDesiredVelocity),
                    )
                    .with_system(
                        update_transform
                            .run_in_state(GameState::Playing)
                            .after(MovementLabels::Kinematics),
                    ),
            );
    }
}

type Uninitialized<'w, 's> =
    Query<'w, 's, (Entity, &'static Transform), (With<MovableSolid>, Without<Movement>)>;

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
enum MovementLabels {
    UpdateDesiredVelocity,
    Kinematics,
}

#[derive(Component, Default)]
struct Movement {
    /// Ideal velocity induced by a global path plan.
    desired: Vec3,
}

impl Movement {
    fn desired_velocity(&self) -> Vec3 {
        self.desired
    }

    fn stop(&mut self) {
        self.desired = Vec3::ZERO;
    }

    fn set_desired_velocity(&mut self, velocity: Vec3) {
        self.desired = velocity;
    }
}

#[derive(Component)]
struct Kinematics {
    /// Velocity during the last update.
    previous: Vec3,
    /// Current velocity.
    current: Vec3,
    /// Current speed in meters per second.
    speed: f32,
    /// Current object heading in radians.
    heading: f32,
}

impl Kinematics {
    fn speed(&self) -> f32 {
        self.speed
    }

    fn heading(&self) -> f32 {
        self.heading
    }

    /// Returns mean velocity over the last frame duration.
    fn frame_velocity(&self) -> Vec3 {
        self.current.lerp(self.previous, 0.5)
    }

    /// This method should be called once every update.
    fn tick(&mut self) {
        self.previous = self.current;
    }

    fn update(&mut self, speed_delta: f32, heading_delta: f32) {
        debug_assert!(speed_delta.is_finite());
        self.speed += speed_delta;
        debug_assert!(heading_delta.is_finite());
        self.heading = normalize_angle(self.heading + heading_delta);
        let (sin, cos) = self.heading.sin_cos();
        self.current = Vec2::new(self.speed * cos, self.speed * sin).to_msl();
    }
}

impl From<&Transform> for Kinematics {
    fn from(transform: &Transform) -> Self {
        Self {
            previous: Vec3::ZERO,
            current: Vec3::ZERO,
            speed: 0.,
            heading: normalize_angle(transform.rotation.to_euler(EulerRot::YXZ).0),
        }
    }
}

fn setup_entities(mut commands: Commands, objects: Uninitialized) {
    for (entity, transform) in objects.iter() {
        commands
            .entity(entity)
            .insert(Movement::default())
            .insert(Kinematics::from(transform));
    }
}

fn update_desired_velocity(
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

fn kinematics(time: Res<Time>, mut objects: Query<(&Movement, &mut Kinematics)>) {
    let time_delta = time.delta_seconds();

    objects.par_for_each_mut(512, |(movement, mut kinematics)| {
        kinematics.tick();

        let desired_velocity = movement.desired_velocity().to_flat();
        let desired_heading = if desired_velocity == Vec2::ZERO {
            kinematics.heading()
        } else {
            desired_velocity.y.atan2(desired_velocity.x)
        };

        let heading_diff = normalize_angle(desired_heading - kinematics.heading());
        let max_heading_delta = MAX_ANGULAR_SPEED * time_delta;
        let heading_delta = heading_diff.clamp(-max_heading_delta, max_heading_delta);

        let max_speed_delta = MAX_ACCELERATION * time_delta;
        let speed_delta = if (heading_diff - heading_delta).abs() > FRAC_PI_4 {
            // Slow down if not going in roughly good direction.
            -kinematics.speed()
        } else {
            desired_velocity.length() - kinematics.speed()
        }
        .clamp(-max_speed_delta, max_speed_delta);

        kinematics.update(speed_delta, heading_delta);
    });
}

fn update_transform(time: Res<Time>, mut objects: Query<(&Kinematics, &mut Transform)>) {
    let time_delta = time.delta_seconds();
    for (kinematics, mut transform) in objects.iter_mut() {
        transform.translation += time_delta * kinematics.frame_velocity();
        transform.rotation = Quat::from_rotation_y(kinematics.heading());
    }
}

fn normalize_angle(mut angle: f32) -> f32 {
    angle %= TAU;
    if angle > PI {
        angle -= TAU;
    } else if angle <= -PI {
        angle += TAU
    }
    angle
}
