use glam::Vec2;

use crate::obstacles::Disc;

pub(crate) struct Obstacle {
    disc: MovingDisc,
    active: bool,
}

impl Obstacle {
    /// Returns a new obstacle.
    ///
    /// # Arguments
    ///
    /// * `active` - whether the obstacle is active, id est whether it is
    ///   actively avoiding obstacles itself.
    pub(crate) fn new(disc: MovingDisc, active: bool) -> Self {
        Self { disc, active }
    }

    pub(super) fn disc(&self) -> &MovingDisc {
        &self.disc
    }

    pub(super) fn active(&self) -> bool {
        self.active
    }
}

/// Description of a moving disc with constant velocity vector.
pub(crate) struct MovingDisc {
    disc: Disc,
    velocity: Vec2,
}

impl MovingDisc {
    pub(crate) fn new(disc: Disc, velocity: Vec2) -> Self {
        Self { disc, velocity }
    }

    pub(super) fn disc(&self) -> Disc {
        self.disc
    }

    pub(super) fn velocity(&self) -> Vec2 {
        self.velocity
    }
}
