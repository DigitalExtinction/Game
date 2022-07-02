use bevy::prelude::*;
use de_core::objects::{ActiveObjectType, BuildingType, UnitType};
use enum_map::{enum_map, EnumMap};

pub(crate) struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InitialHealths>();
    }
}

/// Initial health of spawned objects.
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
                ActiveObjectType::Building(BuildingType::Base) => Health::new(10_000.),
                ActiveObjectType::Building(BuildingType::PowerHub) => Health::new(1000.),
                ActiveObjectType::Unit(UnitType::Attacker) => Health::new(100.),
            },
        }
    }
}

#[derive(Clone, Component)]
pub struct Health {
    health: f32,
}

impl Health {
    const fn new(health: f32) -> Self {
        Self { health }
    }

    pub fn hit(&mut self, damage: f32) {
        self.health = 0f32.max(self.health - damage);
    }

    pub fn destroyed(&self) -> bool {
        self.health <= 0.
    }
}
