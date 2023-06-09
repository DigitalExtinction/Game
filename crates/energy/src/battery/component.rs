use bevy::prelude::*;

use crate::battery::{MIN_ATTACK_ENERGY, MIN_MOVE_ENERGY};

/// The battery component is used to store the energy level of an entity.
///
/// # fields
/// - `capacity`: The maximum capacity of the battery in kilojoules.
/// - `energy`: The current energy level of the battery in kilojoules.
#[derive(Component, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Battery {
    capacity: f32, // in kilojoules
    energy: f32,   // in kilojoules
}

impl Battery {
    pub fn new(capacity: f32, energy: f32) -> Self {
        debug_assert!(capacity.is_finite());
        debug_assert!(capacity > 0.);
        debug_assert!(energy.is_finite());
        debug_assert!(energy >= 0.);
        debug_assert!(energy <= capacity);

        Self { capacity, energy }
    }

    /// The maximum capacity of the battery in kilojoules.
    pub fn capacity(&self) -> f32 {
        self.capacity
    }

    /// The current energy level of the battery in kilojoules.
    pub fn energy(&self) -> f32 {
        self.energy
    }

    /// Does the battery contain enough energy to move?
    pub fn can_move(&self) -> bool {
        self.energy >= MIN_MOVE_ENERGY
    }

    /// Does the battery contain enough energy to attack?
    pub fn can_fire(&self) -> bool {
        self.energy >= MIN_ATTACK_ENERGY
    }

    /// Directly sets the energy level of the battery.
    pub fn set_energy(&mut self, energy: f32) {
        debug_assert!(energy.is_finite());
        debug_assert!(energy >= 0.);
        debug_assert!(energy <= self.capacity);

        self.energy = energy;
    }

    /// Tries to discharge the battery by the given amount of energy.
    ///
    /// # Returns
    ///
    /// `true` if the battery was discharged, `false` otherwise.
    pub fn try_discharge(&mut self, energy: f32) -> bool {
        debug_assert!(energy.is_finite());
        debug_assert!(energy >= 0.);

        if energy >= self.energy {
            return false;
        }

        self.discharge(energy);
        true
    }

    /// Directly discharges the battery by the given amount of energy.
    pub fn discharge(&mut self, energy: f32) {
        debug_assert!(energy.is_finite());
        debug_assert!(energy >= 0.);
        debug_assert!(energy <= self.energy);

        self.energy -= energy;
    }

    /// Tries to charge the battery by the given amount of energy.
    ///
    /// # Returns
    ///
    /// `true` if the battery was charged, `false` otherwise.
    pub fn try_charge(&mut self, energy: f32) -> bool {
        debug_assert!(energy.is_finite());
        debug_assert!(energy >= 0.);

        if energy >= self.capacity - self.energy {
            return false;
        }

        self.charge(energy);
        true
    }

    /// Directly charges the battery by the given amount of energy.
    pub fn charge(&mut self, energy: f32) {
        debug_assert!(energy.is_finite());
        debug_assert!(energy >= 0.);
        debug_assert!(energy <= self.capacity - self.energy);

        self.energy += energy;
    }

    /// Get percentage of energy remaining in the battery.
    pub fn energy_percentage(&self) -> f32 {
        self.energy / self.capacity
    }
}
