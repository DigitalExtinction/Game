mod altitude;
mod cache;
mod disc;
mod kinematics;
mod movement;
mod obstacles;
mod pathing;
mod repulsion;

use std::f32::consts::PI;

use altitude::AltitudePlugin;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use kinematics::KinematicsPlugin;
use movement::MovementPlugin;
use obstacles::ObstaclesPlugin;
use pathing::PathingPlugin;
use repulsion::RepulsionPlugin;

/// Maximum object horizontal speed in meters per second.
const MAX_H_SPEED: f32 = 10.;
/// Maximum object vertical ascending / descending rate in meters per second.
const MAX_V_SPEED: f32 = 4.;
/// Maximum object acceleration in meters per second squared.
const MAX_H_ACCELERATION: f32 = 2. * MAX_H_SPEED;
/// Gravitational acceleration in meters per second squared.
const G_ACCELERATION: f32 = 9.8;
/// Maximum upwards acceleration in meters per second squared.
const MAX_V_ACCELERATION: f32 = 0.5 * G_ACCELERATION;
/// Maximum object angular velocity in radians per second.
const MAX_ANGULAR_SPEED: f32 = PI;
/// Maximum altitude in meters (note that this is not height).
const MAX_ALTITUDE: f32 = 100.;

pub struct MovementPluginGroup;

impl PluginGroup for MovementPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(MovementPlugin)
            .add(PathingPlugin)
            .add(ObstaclesPlugin)
            .add(RepulsionPlugin)
            .add(KinematicsPlugin)
            .add(AltitudePlugin)
    }
}
