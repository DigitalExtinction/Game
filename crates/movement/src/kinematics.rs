use std::f32::consts::{FRAC_PI_4, PI, TAU};

use bevy::prelude::*;
use de_core::{objects::MovableSolid, projection::ToMsl, stages::GameStage, state::GameState};
use iyes_loopless::prelude::*;

use crate::{
    movement::{DesiredMovement, MovementLabels, ObjectVelocity},
    repulsion::RepulsionLables,
    MAX_ACCELERATION, MAX_ANGULAR_SPEED, MAX_SPEED,
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
            SystemSet::new().with_system(
                kinematics
                    .run_in_state(GameState::Playing)
                    .label(KinematicsLabels::Kinematics)
                    .before(MovementLabels::UpdateTransform)
                    .after(RepulsionLables::Apply),
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

    fn update_speed(&mut self, delta: f32) {
        debug_assert!(delta.is_finite());
        self.speed = (self.speed + delta).clamp(0., MAX_SPEED);
    }

    fn update_heading(&mut self, delta: f32) {
        debug_assert!(delta.is_finite());
        self.heading = normalize_angle(self.heading + delta);
    }

    fn compute_velocity(&self) -> Vec3 {
        let (sin, cos) = self.heading.sin_cos();
        Vec2::new(self.speed * cos, self.speed * sin).to_msl()
    }
}

impl From<&Transform> for Kinematics {
    fn from(transform: &Transform) -> Self {
        Self {
            speed: 0.,
            heading: normalize_angle(transform.rotation.to_euler(EulerRot::YXZ).0),
        }
    }
}

fn setup_entities(mut commands: Commands, objects: Uninitialized) {
    for (entity, transform) in objects.iter() {
        commands.entity(entity).insert(Kinematics::from(transform));
    }
}

fn kinematics(
    time: Res<Time>,
    mut objects: Query<(&DesiredMovement, &mut Kinematics, &mut ObjectVelocity)>,
) {
    let time_delta = time.delta_seconds();

    objects.par_for_each_mut(512, |(movement, mut kinematics, mut velocity)| {
        let desired_velocity = movement.velocity();
        let desired_heading = if desired_velocity == Vec2::ZERO {
            kinematics.heading()
        } else {
            desired_velocity.y.atan2(desired_velocity.x)
        };

        let heading_diff = normalize_angle(desired_heading - kinematics.heading());
        let max_heading_delta = MAX_ANGULAR_SPEED * time_delta;
        let heading_delta = heading_diff.clamp(-max_heading_delta, max_heading_delta);
        kinematics.update_heading(heading_delta);

        let max_speed_delta = MAX_ACCELERATION * time_delta;
        let speed_delta = if (heading_diff - heading_delta).abs() > FRAC_PI_4 {
            // Slow down if not going in roughly good direction.
            -kinematics.speed()
        } else {
            desired_velocity.length() - kinematics.speed()
        }
        .clamp(-max_speed_delta, max_speed_delta);

        kinematics.update_speed(speed_delta);
        velocity.update(kinematics.compute_velocity(), kinematics.heading());
    });
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
