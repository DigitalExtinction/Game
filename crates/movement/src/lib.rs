mod kinematics;
mod movement;
mod pathing;

use std::f32::consts::PI;

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use kinematics::KinematicsPlugin;
use movement::MovementPlugin;
use pathing::PathingPlugin;

/// Maximum object speed in meters per second.
const MAX_SPEED: f32 = 10.;
/// Maximum object acceleration in meters per second squared.
const MAX_ACCELERATION: f32 = 2. * MAX_SPEED;
/// Maximum object angular velocity in radians per second.
const MAX_ANGULAR_SPEED: f32 = PI;

pub struct MovementPluginGroup;

impl PluginGroup for MovementPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(MovementPlugin)
            .add(PathingPlugin)
            .add(KinematicsPlugin);
    }
}
