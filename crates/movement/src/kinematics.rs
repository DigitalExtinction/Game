use std::f32::consts::{FRAC_PI_4, PI, TAU};

use bevy::prelude::*;
use de_core::{objects::MovableSolid, projection::ToMsl, stages::GameStage, state::GameState};
use iyes_loopless::prelude::*;

use crate::{
    movement::DesiredMovement, repulsion::RepulsionLables, MAX_ACCELERATION,
    MAX_ANGULAR_ACCELERATION, MAX_ANGULAR_SPEED, MAX_SPEED,
};

pub(crate) struct KinematicsPlugin;

impl Plugin for KinematicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            GameStage::PreMovement,
            setup_entities.run_in_state(GameState::Playing),
        )
        .add_system_set_to_stage(
            GameStage::Movement,
            SystemSet::new()
                .with_system(
                    kinematics
                        .run_in_state(GameState::Playing)
                        .label(KinematicsLabels::Kinematics)
                        .after(RepulsionLables::Apply),
                )
                .with_system(
                    update_transform
                        .run_in_state(GameState::Playing)
                        .after(KinematicsLabels::Kinematics),
                ),
        );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
enum KinematicsLabels {
    Kinematics,
}

type Uninitialized<'w, 's> =
    Query<'w, 's, (Entity, &'static Transform), (With<MovableSolid>, Without<Kinematics>)>;

#[derive(Component)]
struct Kinematics {
    /// Velocity during the last update.
    previous: Vec3,
    /// Current velocity.
    current: Vec3,
    /// Current speed in meters per second.
    speed: f32,
    /// Rotation of the object in radians per second.
    angular_velocity: f32,
    /// Current object heading in radians.
    heading: f32,
}

impl Kinematics {
    fn speed(&self) -> f32 {
        self.speed
    }

    fn angular_velocity(&self) -> f32 {
        self.angular_velocity
    }

    fn update_angular_velocity(&mut self, delta: f32) {
        self.angular_velocity += delta
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
        self.speed = (self.speed + speed_delta).clamp(0., MAX_SPEED);
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
            angular_velocity: 0.,
            heading: normalize_angle(transform.rotation.to_euler(EulerRot::YXZ).0),
        }
    }
}

fn setup_entities(mut commands: Commands, objects: Uninitialized) {
    for (entity, transform) in objects.iter() {
        commands.entity(entity).insert(Kinematics::from(transform));
    }
}

fn kinematics(time: Res<Time>, mut objects: Query<(&DesiredMovement, &mut Kinematics)>) {
    let time_delta = time.delta_seconds();

    objects.par_for_each_mut(512, |(movement, mut kinematics)| {
        kinematics.tick();

        let desired_velocity = movement.velocity();
        let desired_heading = if desired_velocity == Vec2::ZERO {
            kinematics.heading()
        } else {
            desired_velocity.y.atan2(desired_velocity.x)
        };

        let heading_diff = normalize_angle(desired_heading - kinematics.heading());

        // TODO proper variable names
        // TODO speed vs velocity
        let a = (2. * MAX_ANGULAR_ACCELERATION * heading_diff).abs().sqrt();
        let b = a.min(MAX_ANGULAR_SPEED);
        let desired_angular_velocity = heading_diff.signum() * b;
        let max_angular_velocity_delta = MAX_ANGULAR_ACCELERATION * time_delta;
        let angular_velocity_delta = (desired_angular_velocity - kinematics.angular_velocity())
            .clamp(-max_angular_velocity_delta, max_angular_velocity_delta);
        kinematics.update_angular_velocity(angular_velocity_delta);
        let heading_delta = time_delta * kinematics.angular_velocity();

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
