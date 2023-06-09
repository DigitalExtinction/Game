use std::{cmp::Ordering, time::Duration};

use bevy::prelude::Component;
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone)]
pub struct LaserCannon {
    muzzle: Vec3,
    range: f32,
    damage: f32,
    charge: LaserCharge,
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

    pub fn charge(&self) -> &LaserCharge {
        &self.charge
    }

    pub fn charge_mut(&mut self) -> &mut LaserCharge {
        &mut self.charge
    }
}

/// Charge of a laser cannon. It is used to keep track of needed cannon
/// charging time.
///
/// A laser cannon cannot fire immediately after it is activated, but takes
/// time to charge. After firing, it has to (re)charge. It has to recharge
/// after it any (re)activation.
///
/// [`Self::tick`] must be called during every frame. After that,
/// [`Self::hold`] or [`Self::fire`] must be called in a loop while
/// [`Self::charged`] returns true.
///
/// LaserTimer implements total ordering based on elapsed time since reaching
/// charge for at least one fire.
#[derive(Clone, PartialEq)]
pub struct LaserCharge {
    charge_time: Duration,
    discharge_time: Duration,
    charge: f32,
}

impl LaserCharge {
    /// Returns a new timer.
    ///
    /// # Arguments
    ///
    /// * `charge_time` - time it takes to fully (re)charge the laser cannon.
    ///
    /// * `discharge_time` - time it takes to fully discharge the laser cannot
    ///   if it is not actively charged.
    ///
    /// # Panics
    ///
    /// Panics if `charge_time` or `discharge_time` spans zero time.
    fn new(charge_time: Duration, discharge_time: Duration) -> Self {
        assert!(!charge_time.is_zero());
        assert!(!discharge_time.is_zero());

        Self {
            charge_time,
            discharge_time,
            charge: 0.,
        }
    }

    /// Updates the timer.
    ///
    /// # Arguments
    ///
    /// * `time_delta` - time delta since last call to this method.
    ///
    /// * `charge` - true if the cannon is charging, false if it is
    ///   discharging.
    pub fn tick(&mut self, time_delta: Duration, charge: bool) {
        if charge {
            self.charge += time_delta.as_secs_f32() / self.charge_time.as_secs_f32();
        } else {
            self.charge -= time_delta.as_secs_f32() / self.discharge_time.as_secs_f32();
            self.charge = self.charge.max(0.);
        }
    }

    /// Returns true if the cannon is charged for at least one fire.
    pub fn charged(&self) -> bool {
        self.charge >= 1.
    }

    /// Clamps charge to one fire.
    ///
    /// Must be called after [`Self::tick`].
    pub fn hold(&mut self) {
        self.charge = self.charge.min(1.);
    }

    /// Subtracts one fire worth of charge and returns true if there is charge
    /// for another fire.
    ///
    /// Must be called after [`Self::tick`].
    pub fn fire(&mut self) -> bool {
        debug_assert!(self.charge >= 1.);
        self.charge -= 1.;
        self.charged()
    }
}

impl Eq for LaserCharge {}

impl Ord for LaserCharge {
    fn cmp(&self, other: &Self) -> Ordering {
        // Cannon is either fired until the charge goes below 1. or its charge
        // is clipped at 1. Therefore, if the cannon is to be fired, it must
        // have been charging since the last tick.
        //
        // The ordering (who comes first) is based on the above assumptions.
        let self_over = (self.charge - 1.) * self.charge_time.as_secs_f32();
        let other_over = (other.charge - 1.) * other.charge_time.as_secs_f32();
        self_over.partial_cmp(&other_over).unwrap()
    }
}

impl PartialOrd for LaserCharge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl TryFrom<LaserCannonSerde> for LaserCannon {
    type Error = anyhow::Error;

    fn try_from(info: LaserCannonSerde) -> Result<Self, Self::Error> {
        Ok(Self {
            muzzle: Vec3::from_slice(info.muzzle.as_slice()),
            range: info.range,
            damage: info.damage,
            charge: LaserCharge::new(
                Duration::from_secs_f32(info.charge_time_sec),
                Duration::from_secs_f32(info.discharge_time_sec),
            ),
        })
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct LaserCannonSerde {
    muzzle: [f32; 3],
    range: f32,
    damage: f32,
    charge_time_sec: f32,
    discharge_time_sec: f32,
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::*;

    #[test]
    fn test_charge() {
        let mut charge =
            LaserCharge::new(Duration::from_secs_f32(2.5), Duration::from_secs_f32(3.5));

        assert!(!charge.charged());

        charge.tick(Duration::from_secs(2), true);
        assert!(!charge.charged()); // charge: 0.8
        charge.tick(Duration::from_secs(1), true);
        assert!(charge.charged()); // charge: 1.2
        charge.fire();
        assert!(!charge.charged()); // 0.2
        charge.tick(Duration::from_secs(2), true);
        assert!(charge.charged()); // charge: 1
        charge.fire();
        assert!(!charge.charged()); // charge: 0
        charge.tick(Duration::from_secs_f32(2.4), true);
        assert!(!charge.charged()); // charge: 0.96
        charge.tick(Duration::from_secs_f32(2.6), true);
        assert!(charge.charged()); // charge: 2
        charge.tick(Duration::from_secs_f32(3.4), false);
        assert!(charge.charged()); // charge: 1.028
        charge.tick(Duration::from_secs_f32(0.15), false);
        assert!(!charge.charged()); // charge: 0.985
    }

    #[test]
    fn test_timer_ordering() {
        let mut a = LaserCharge::new(Duration::from_secs(2), Duration::from_secs(1));
        a.tick(Duration::from_secs(3), true);
        let mut b = LaserCharge::new(Duration::from_secs(4), Duration::from_secs(1));
        b.tick(Duration::from_secs(5), true);
        let mut c = LaserCharge::new(Duration::from_secs_f32(0.1), Duration::from_secs(1));
        c.tick(Duration::from_secs(10), true);

        assert!(a.cmp(&b) == Ordering::Equal);
        assert!(b.cmp(&a) == Ordering::Equal);
        assert!(a.cmp(&c) == Ordering::Less);
        assert!(c.cmp(&a) == Ordering::Greater);
        assert!(b.cmp(&c) == Ordering::Less);
        assert!(c.cmp(&b) == Ordering::Greater);
    }
}
