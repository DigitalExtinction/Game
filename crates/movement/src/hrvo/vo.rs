use glam::{IVec2, Vec2};
use tinyvec::ArrayVec;

use super::{
    edge::Edge,
    line::{Line, LineLineIntersection},
    obstacle::{MovingDisc, Obstacle},
    region::Region,
    scale::{vec_div_to_scale, vec_to_scale},
};
use crate::hrvo::{bounds::Bounds, line::Signum, scale::vec_from_scale};

const DISTANCE_EPS: f32 = 0.01;
const MIN_COS_TRIM: f32 = 0.5;
const MAX_DELTA_T: f32 = 5.;

/// Compute region of collision velocities.
///
/// The region is Hybrid Reciprocal Velocity Obstacle in case the obstacle is
/// active. Otherwise, it is plain Velocity Obstacle.
///
/// # Arguments
///
/// `active` - collision velocities of this object are considered.
///
/// `obstacle` - obstacle which induces the velocity obstacle region.
///
/// `max_speed_squared` - square of maximum allowed speed of `active`.
pub(super) fn compute_region(
    active: &MovingDisc,
    obstacle: &Obstacle,
    max_speed: f32,
) -> Option<Region> {
    let secondary_disc = obstacle.disc().disc();
    let relative_position_world = secondary_disc.center() - active.disc().center();
    let distance_world = relative_position_world.length();
    if distance_world <= DISTANCE_EPS {
        return None;
    }

    // Unit vector of the midline of the (HR)VO cone.
    let midline_world = relative_position_world / distance_world;
    let radius_world = active.disc().radius() + secondary_disc.radius();
    debug_assert!(radius_world > 0.);

    // Below is sine and cosine of the angle between midline of the cone
    // and its side.
    let sin: f32 = if distance_world > radius_world {
        let sin = radius_world / distance_world;
        sin
    } else {
        // Due to latency and multiple other effects, object might temporarily
        // become closer than the sum of radii (i.e. their disks intersecting).
        1.
    };
    debug_assert!(sin > 0.);
    debug_assert!(sin <= 1.);
    let cos = (1. - sin.powi(2)).sqrt();

    let gap = distance_world - radius_world;
    let repulsion = if gap >= max_speed {
        0.
    } else if gap <= 0. {
        0.9 * max_speed
    } else {
        gap.powi(-2).min(0.9 * max_speed)
    };
    let apex_shift = -vec_to_scale(repulsion * midline_world);

    let left_world = Vec2::new(
        cos * midline_world.x - sin * midline_world.y,
        sin * midline_world.x + cos * midline_world.y,
    );
    let left_scale = vec_to_scale(left_world);
    let right_world = Vec2::new(
        cos * midline_world.x + sin * midline_world.y,
        -sin * midline_world.x + cos * midline_world.y,
    );
    let right_scale = vec_to_scale(right_world);

    let apex_scale = apex_shift
        + if obstacle.active() {
            match compute_hrvo_apex(
                active,
                obstacle.disc(),
                midline_world,
                left_scale,
                right_scale,
            ) {
                Some(apex) => apex,
                None => return None,
            }
        } else {
            vec_to_scale(obstacle.disc().velocity())
        };

    // TODO cut apex (do not avoid collisions in long future)
    // TODO enlarge to sides

    let apex_world = vec_from_scale(apex_scale);

    let max_speed_squared = max_speed.powi(2);
    let mut edges: ArrayVec<[Edge; 3]> = ArrayVec::from([
        Edge::new(
            Line::new(apex_scale, left_scale),
            Signum::Negative,
            i32::MAX,
            Bounds::compute(apex_world, left_world, max_speed_squared),
        ),
        Edge::new(
            Line::new(apex_scale, right_scale),
            Signum::Positive,
            i32::MAX,
            Bounds::compute(apex_world, right_world, max_speed_squared),
        ),
        Edge::default(),
    ]);
    edges.set_len(2);
    Some(Region::new(edges))
}

fn compute_hrvo_apex(
    active: &MovingDisc,
    secondary: &MovingDisc,
    midline: Vec2,
    left: IVec2,
    right: IVec2,
) -> Option<IVec2> {
    let rvo_apex_world = (active.velocity() + secondary.velocity()) / 2.;
    let side_world = midline.perp_dot(active.velocity() - rvo_apex_world);

    // TODO use a constant for minimum side
    if side_world.abs() < 0.01 {
        // VO is used since either secondary object is passive or the velocity
        // is close to the RVO midline.
        return Some(vec_to_scale(secondary.velocity()));
    }

    let (left_point, right_point) = if side_world < 0. {
        // Primary velocity is on the right side of RVO, thus we enlarge the
        // left side of RVO to get HRVO.
        (secondary.velocity(), rvo_apex_world)
    } else {
        (rvo_apex_world, secondary.velocity())
    };

    let left_line = Line::new(vec_to_scale(left_point), left);
    let right_line = Line::new(vec_to_scale(right_point), right);

    match left_line.intersection(right_line) {
        None => return None, // This might happen when the cone is too narrow.
        Some(intersection) => {
            let intersection = match intersection {
                // This might happen when the cone is too narrow.
                LineLineIntersection::Coincidental => return None,
                LineLineIntersection::Point(intersection) => intersection,
            };

            Some(left_line.point() + vec_div_to_scale(intersection.primary_parameter() * left))
        }
    }
}

// TODO test that frustum left/right are ~1024 long

#[cfg(test)]
mod tests {}
