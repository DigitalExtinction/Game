use std::{cmp::Ordering, time::Duration};

use bevy::prelude::Component;
use glam::Vec3;

use crate::loader::LaserCannonInfo;

#[derive(Component, Clone)]
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

    /// When an object is hit, its health is decreased by this amount.
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

/// Timer of a laser cannon. It is used to keep track of needed cannon charging
/// time.
///
/// A laser cannon cannot fire immediately after it is activated, but takes
/// time to charge. After firing, it has to (re)charge. It has to recharge
/// after it any (re)activation.
///
/// LaserTimer implements total ordering based on elapsed time.
#[derive(Eq, Clone)]
pub struct LaserTimer {
    interval: Duration,
    elapsed: Duration,
}

impl LaserTimer {
    /// Returns a new timer.
    ///
    /// # Arguments
    ///
    /// * `interval` - time it takes to (re)charge the laser cannon.
    fn new(interval: Duration) -> Self {
        Self {
            interval,
            elapsed: Duration::new(0, 0),
        }
    }

    /// Updates the timer. This must be called during every frame.
    pub fn tick(&mut self, tick: Duration) {
        self.elapsed += tick;
    }

    /// Resets the (re)charge timer. This must be called during every frame
    /// when the cannon is not activated.
    pub fn reset(&mut self) {
        self.elapsed = Duration::new(0, 0);
    }

    /// Returns true if the cannon is ready to fire and updates the timer for
    /// (re)charging.
    ///
    /// It is assumed that the laser cannon is fired after this method returns
    /// true.
    ///
    /// The timer keeps track of any extra time beyond the time to (re)charge
    /// so that its function is not dependent on update rate. However, update
    /// interval (time between to successive calls to this method) must be
    /// smaller or equal to laser charging interval.
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
        self.elapsed.cmp(&other.elapsed)
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

impl From<&LaserCannonInfo> for LaserCannon {
    fn from(info: &LaserCannonInfo) -> Self {
        Self {
            muzzle: Vec3::from_slice(info.muzzle().as_slice()),
            range: info.range(),
            damage: info.damage(),
            timer: LaserTimer::new(Duration::from_secs_f32(info.recharge_interval())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::*;

    #[test]
    fn test_timer() {
        let mut timer = LaserTimer::new(Duration::from_secs_f32(2.5));

        assert!(!timer.check_and_update());
        assert!(!timer.check_and_update());
        timer.tick(Duration::from_secs(2));
        assert!(!timer.check_and_update());
        timer.tick(Duration::from_secs(1));
        assert!(timer.check_and_update());
        assert!(!timer.check_and_update());
        timer.tick(Duration::from_secs(2));
        assert!(timer.check_and_update());
        assert!(!timer.check_and_update());

        timer.tick(Duration::from_secs(100));
        timer.reset();
        assert!(!timer.check_and_update());
    }

    #[test]
    fn test_timer_ordering() {
        let mut a = LaserTimer::new(Duration::from_secs(2));
        a.tick(Duration::from_secs(3));
        let mut b = LaserTimer::new(Duration::from_secs(3));
        b.tick(Duration::from_secs(3));
        let mut c = LaserTimer::new(Duration::from_secs(0));
        c.tick(Duration::from_secs(10));

        assert!(a.cmp(&b) == Ordering::Equal);
        assert!(b.cmp(&a) == Ordering::Equal);
        assert!(a.cmp(&c) == Ordering::Less);
        assert!(c.cmp(&a) == Ordering::Greater);
        assert!(b.cmp(&c) == Ordering::Less);
        assert!(c.cmp(&b) == Ordering::Greater);
    }
}
