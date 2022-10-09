//! This module implements Hybrid Reciprocal Velocity Obstacle (HRVO) algorithm
//! loosely based on paper:
//!
//! Snape, Jamie, et al. "The hybrid reciprocal velocity obstacle." IEEE
//! Transactions on Robotics 27.4 (2011): 696-706.

use bevy::prelude::*;
use de_core::{projection::ToFlat, stages::GameStage, state::GameState};
use glam::Vec2;
use iyes_loopless::prelude::*;
pub(crate) use obstacle::{MovingDisc, Obstacle};

use self::{
    candidates::Candidates,
    region::Region,
    scale::{vec_from_scale, vec_to_scale},
    vo::compute_region,
};
use crate::{
    cache::DecayingCache,
    movement::{DesiredMovement, RealMovement},
    obstacles::{Disc, MovableObstacles, ObstaclesLables},
    pathing::PathingLabels,
    MAX_SPEED,
};

mod bounds;
mod candidates;
mod edge;
mod line;
mod obstacle;
mod parameters;
mod region;
mod scale;
mod vo;

pub(crate) struct HrvoPlugin;

impl Plugin for HrvoPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            GameStage::Movement,
            avoid_obstacles
                .run_in_state(GameState::Playing)
                .label(HrvoLabels::AvoidObstacles)
                .after(ObstaclesLables::UpdateNearby)
                .after(PathingLabels::FollowPath),
        );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum HrvoLabels {
    AvoidObstacles,
}

fn avoid_obstacles(
    mut objects: Query<(
        &Disc,
        &mut DesiredMovement,
        &RealMovement,
        &DecayingCache<MovableObstacles>,
    )>,
    others: Query<(&Disc, &DesiredMovement, &RealMovement)>,
) {
    objects.par_for_each_mut(
        512,
        |(&disc, mut desired_movement, real_movement, obstacles)| {
            if desired_movement.stopped() {
                return;
            }

            let disc = MovingDisc::new(disc, real_movement.current_velocity().to_flat());
            let obstacles: Vec<Obstacle> = obstacles
                .entities()
                .iter()
                .map(|&entity| {
                    let (&disc, desired_movement, real_movement) = others.get(entity).unwrap();
                    Obstacle::new(
                        MovingDisc::new(disc, real_movement.current_velocity().to_flat()),
                        !desired_movement.stopped(),
                    )
                })
                .collect();

            let velocity = hrvo(
                real_movement.current_velocity().to_flat(),
                MAX_SPEED,
                &disc,
                &obstacles,
            );
            desired_movement.update(velocity);
        },
    );
}

pub(crate) fn hrvo(
    desired: Vec2,
    max_speed: f32,
    active: &MovingDisc,
    obstacles: &[Obstacle],
) -> Vec2 {
    let regions: Vec<Region> = obstacles
        .iter()
        .filter_map(|obstacle| compute_region(active, obstacle, max_speed))
        .collect();

    let desired_scaled = vec_to_scale(desired);
    if regions.iter().all(|r| !r.contains(desired_scaled)) {
        return desired;
    }

    let mut best = (Vec2::ZERO, f32::INFINITY);
    for candidate in Candidates::new(desired_scaled, regions.as_slice()) {
        let candidate = vec_from_scale(candidate);
        let distance = candidate.distance_squared(desired);
        if distance < best.1 {
            best = (candidate, distance);
        }
    }

    best.0
}
