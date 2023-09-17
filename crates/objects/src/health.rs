use bevy::prelude::*;
use de_types::objects::{ActiveObjectType, BuildingType, UnitType};
use enum_map::{enum_map, EnumMap};

pub(crate) struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InitialHealths>();
    }
}

/// Initial health of spawned objects.
#[derive(Resource)]
pub struct InitialHealths {
    healths: EnumMap<ActiveObjectType, Health>,
}

impl InitialHealths {
    pub fn health(&self, object_type: ActiveObjectType) -> &Health {
        &self.healths[object_type]
    }
}

impl Default for InitialHealths {
    fn default() -> Self {
        Self {
            healths: enum_map! {
                ActiveObjectType::Building(BuildingType::Base) => Health::full(100.),
                ActiveObjectType::Building(BuildingType::PowerHub) => Health::full(40.),
                ActiveObjectType::Unit(UnitType::Attacker) => Health::full(10.),
            },
        }
    }
}

#[derive(Clone, Component)]
pub struct Health {
    max: f32,
    health: f32,
}

impl Health {
    /// Crates a new health object with a given maximum health. Current health
    /// is set to maximum.
    ///
    /// # Arguments
    ///
    /// * `health` - maximum & current health. Must be a positive finite
    ///   number.
    const fn full(health: f32) -> Self {
        Self {
            max: health,
            health,
        }
    }

    /// Returns the fraction of remaining health, i.e. ratio between current
    /// health and maximum health.
    pub fn fraction(&self) -> f32 {
        debug_assert!(self.health.is_finite());
        debug_assert!(self.max.is_finite());
        debug_assert!(0. <= self.health);
        debug_assert!(self.health <= self.max);

        self.health / self.max
    }

    /// This method decreases health.
    ///
    /// # Arguments
    ///
    /// * `delta` - amount of change, i.e. by how much is the health increased.
    ///
    /// # Panics
    ///
    /// This method might panic if `delta` is not a finite number.
    pub fn update(&mut self, delta: f32) {
        debug_assert!(delta.is_finite());
        self.health = (self.health + delta).clamp(0., self.max);
    }

    pub fn destroyed(&self) -> bool {
        self.health <= 0.
    }
}
