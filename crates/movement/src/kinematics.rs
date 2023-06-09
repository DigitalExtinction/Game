use std::f32::consts::{FRAC_PI_4, PI, TAU};

use bevy::prelude::*;
use de_core::{
    baseset::GameSet, gamestate::GameState, objects::MovableSolid, projection::ToAltitude,
    state::AppState,
};

use crate::{
    altitude::{AltitudeSet, DesiredClimbing},
    movement::{DesiredVelocity, MovementSet, ObjectVelocity},
    repulsion::{RepulsionLables, RepulsionVelocity},
    G_ACCELERATION, MAX_ANGULAR_SPEED, MAX_H_SPEED, MAX_V_ACCELERATION, MAX_V_SPEED,
};

pub(crate) struct KinematicsPlugin;

impl Plugin for KinematicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            setup_entities
                .in_base_set(GameSet::PreMovement)
                .run_if(in_state(AppState::InGame)),
        )
        .add_system(
            kinematics
                .in_base_set(GameSet::Movement)
                .run_if(in_state(GameState::Playing))
                .in_set(KinematicsSet::Kinematics)
                .before(MovementSet::UpdateTransform)
                .after(RepulsionLables::Apply)
                .after(AltitudeSet::Update),
        );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum KinematicsSet {
    Kinematics,
}

type Uninitialized<'w, 's> =
    Query<'w, 's, (Entity, &'static Transform), (With<MovableSolid>, Without<Kinematics>)>;

#[derive(Component)]
struct Kinematics {
    /// Current horizontal speed in meters per second.
    horizontal_speed: f32,
    /// Current vertical speed in meters per second.
    vertical_speed: f32,
    /// Current object heading in radians.
    heading: f32,
}

impl Kinematics {
    fn horizontal_speed(&self) -> f32 {
        self.horizontal_speed
    }

    fn vertical_speed(&self) -> f32 {
        self.vertical_speed
    }

    fn heading(&self) -> f32 {
        self.heading
    }

    fn update_horizontal_speed(&mut self, delta: f32) {
        debug_assert!(delta.is_finite());
        self.horizontal_speed = (self.horizontal_speed + delta).clamp(0., MAX_H_SPEED);
    }

    fn update_vertical_speed(&mut self, delta: f32) {
        debug_assert!(delta.is_finite());
        self.vertical_speed = (self.vertical_speed + delta).clamp(-MAX_V_SPEED, MAX_V_SPEED);
    }

    fn update_heading(&mut self, delta: f32) {
        debug_assert!(delta.is_finite());
        self.heading = normalize_angle(self.heading + delta);
    }

    fn compute_velocity(&self) -> Vec3 {
        let (sin, cos) = self.heading.sin_cos();
        (self.horizontal_speed * Vec2::new(cos, sin)).to_altitude(self.vertical_speed)
    }
}

impl From<&Transform> for Kinematics {
    fn from(transform: &Transform) -> Self {
        Self {
            horizontal_speed: 0.,
            vertical_speed: 0.,
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
    mut objects: Query<(
        &DesiredVelocity<RepulsionVelocity>,
        &DesiredClimbing,
        &mut Kinematics,
        &mut ObjectVelocity,
    )>,
) {
    let time_delta = time.delta_seconds();

    objects
        .par_iter_mut()
        .for_each_mut(|(movement, climbing, mut kinematics, mut velocity)| {
            let desired_h_velocity = movement.velocity();
            let desired_heading = if desired_h_velocity == Vec2::ZERO {
                kinematics.heading()
            } else {
                desired_h_velocity.y.atan2(desired_h_velocity.x)
            };

            let heading_diff = normalize_angle(desired_heading - kinematics.heading());
            let max_heading_delta = MAX_ANGULAR_SPEED * time_delta;
            let heading_delta = heading_diff.clamp(-max_heading_delta, max_heading_delta);
            kinematics.update_heading(heading_delta);

            let max_h_speed_delta = MAX_H_SPEED * time_delta;
            let h_speed_delta = if (heading_diff - heading_delta).abs() > FRAC_PI_4 {
                // Slow down if not going in roughly good direction.
                -kinematics.horizontal_speed()
            } else {
                desired_h_velocity.length() - kinematics.horizontal_speed()
            }
            .clamp(-max_h_speed_delta, max_h_speed_delta);
            kinematics.update_horizontal_speed(h_speed_delta);

            let v_speed_delta = (climbing.speed() - kinematics.vertical_speed()).clamp(
                -time_delta * G_ACCELERATION,
                time_delta * MAX_V_ACCELERATION,
            );
            kinematics.update_vertical_speed(v_speed_delta);

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
