mod cache;
mod disc;
mod kinematics;
mod movement;
mod obstacles;
mod pathing;
mod repulsion;

use std::f32::consts::PI;

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use kinematics::KinematicsPlugin;
use movement::MovementPlugin;
use obstacles::ObstaclesPlugin;
use pathing::PathingPlugin;
use repulsion::RepulsionPlugin;

/// Maximum object speed in meters per second.
const MAX_SPEED: f32 = 10.;
/// Maximum object acceleration in meters per second squared.
const MAX_ACCELERATION: f32 = 2. * MAX_SPEED;
/// Maximum object angular velocity in radians per second.
const MAX_ANGULAR_SPEED: f32 = PI;

pub struct MovementPluginGroup;

impl PluginGroup for MovementPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(MovementPlugin)
            .add(PathingPlugin)
            .add(ObstaclesPlugin)
            .add(RepulsionPlugin)
            .add(KinematicsPlugin)
    }
}
