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
    disc::Disc,
    movement::{add_desired_velocity, DesiredVelocity, ObjectVelocity},
    obstacles::{MovableObstacles, ObstaclesLables},
    pathing::{PathVelocity, PathingLabels},
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
            GameStage::PreMovement,
            add_desired_velocity::<HrvoVelocity>.run_in_state(GameState::Playing),
        )
        .add_system_to_stage(
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

pub(crate) struct HrvoVelocity;

fn avoid_obstacles(
    mut objects: Query<(
        &Disc,
        &DesiredVelocity<PathVelocity>,
        &mut DesiredVelocity<HrvoVelocity>,
        &ObjectVelocity,
        &DecayingCache<MovableObstacles>,
    )>,
    others: Query<(&Disc, &DesiredVelocity<PathVelocity>, &ObjectVelocity)>,
) {
    objects.par_for_each_mut(
        512,
        |(&disc, path_velocity, mut hrvo_velocity, object_velocity, obstacles)| {
            if path_velocity.stationary() {
                hrvo_velocity.update(Vec2::ZERO);
                return;
            }

            let current = object_velocity.current().to_flat();

            let mut obstacles: Vec<(Entity, f32)> = obstacles
                .entities()
                .iter()
                .map(|&entity| {
                    let diff = others.get(entity).unwrap().0.center() - disc.center();
                    let dist = diff.length();
                    let dot = current.dot(diff);

                    // TODO handle division by zero
                    // TODO constant
                    let score = dist - 0.4 * (dot / dist);

                    (entity, score)
                })
                .collect();
            obstacles.sort_unstable_by(|a, b| f32::total_cmp(&a.1, &b.1));

            let obstacles: Vec<Obstacle> = obstacles
                .iter()
                .take(5) // TODO constant
                .map(|&(entity, _)| {
                    let (&disc, path_velocity, object_velocity) = others.get(entity).unwrap();
                    Obstacle::new(
                        MovingDisc::new(disc, object_velocity.current().to_flat()),
                        !path_velocity.stationary(),
                    )
                })
                .collect();

            let disc = MovingDisc::new(disc.inflated(0.1), current);
            let velocity = hrvo(path_velocity.velocity(), MAX_SPEED, &disc, &obstacles);
            hrvo_velocity.update(velocity);
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

    // TODO constant
    let mut best = (desired.clamp_length_max(0.1), f32::INFINITY);
    for candidate in Candidates::new(desired_scaled, regions.as_slice()) {
        let candidate = vec_from_scale(candidate);
        let distance = candidate.distance_squared(desired);
        if distance < best.1 {
            best = (candidate, distance);
        }
    }

    best.0
}
