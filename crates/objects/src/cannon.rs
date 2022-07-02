use std::{cmp::Ordering, time::Duration};

use bevy::prelude::Component;
use glam::Vec3;

use crate::loader::LaserCannonInfo;

#[derive(Component)]
pub struct LaserCannon {
    muzzle: Vec3,
    range: f32,
    damage: f32,
    timer: LaserTimer,
}

impl LaserCannon {
    /// Returns relative position of the cannon muzzle to the object.
    pub fn muzzle(&self) -> Vec3 {
        self.muzzle
    }

    /// Maximum range of the laser in meters. Objects further than this cannot
    /// be hit.
    pub fn range(&self) -> f32 {
        self.range
    }

    /// Health decrease done to a hit object.
    pub fn damage(&self) -> f32 {
        self.damage
    }

    pub fn timer(&self) -> &LaserTimer {
        &self.timer
    }

    pub fn timer_mut(&mut self) -> &mut LaserTimer {
        &mut self.timer
    }
}

#[derive(Eq)]
pub struct LaserTimer {
    interval: Duration,
    elapsed: Duration,
}

impl LaserTimer {
    fn new(interval: Duration) -> Self {
        Self {
            interval,
            elapsed: Duration::new(0, 0),
        }
    }

    // TODO docs
    pub fn tick(&mut self, tick: Duration) {
        self.elapsed += tick;
    }

    pub fn reset(&mut self) {
        self.elapsed = Duration::new(0, 0);
    }

    // TODO doc
    pub fn check_and_update(&mut self) -> bool {
        if self.elapsed >= self.interval {
            self.elapsed -= self.interval;
            true
        } else {
            false
        }
    }
}

impl Ord for LaserTimer {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.elapsed - self.interval).cmp(&(other.elapsed - other.interval))
    }
}

impl PartialOrd for LaserTimer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for LaserTimer {
    fn eq(&self, other: &Self) -> bool {
        self.elapsed == other.elapsed && self.interval == other.interval
    }
}

impl From<LaserCannonInfo> for LaserCannon {
    fn from(info: LaserCannonInfo) -> Self {
        Self {
            muzzle: Vec3::from_slice(info.muzzle().as_slice()),
            range: info.range(),
            damage: info.damage(),
            timer: LaserTimer::new(Duration::from_secs_f32(info.recharge_interval())),
        }
    }
}
