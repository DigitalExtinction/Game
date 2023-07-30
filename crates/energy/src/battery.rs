use bevy::prelude::*;
use de_core::objects::Active;

pub(crate) struct BatteryPlugin;

impl Plugin for BatteryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, discharge_battery)
            .add_systems(PostUpdate, spawn_battery);
    }
}

/// The rate at which the battery discharges in Joules per second.
const DISCHARGE_RATE: f64 = 30_000.;
/// The default capacity of the battery in Joules.
const DEFAULT_CAPACITY: f64 = 100_000_000.; // 100 Mj

/// The battery component is used to store the energy level of an entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Battery {
    /// The maximum capacity of the battery in joules.
    capacity: f64,

    /// The current energy level of the battery in joules.
    energy: f64,
}

impl Default for Battery {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY, DEFAULT_CAPACITY)
    }
}

impl Battery {
    fn new(capacity: f64, energy: f64) -> Self {
        debug_assert!(capacity.is_finite());
        debug_assert!(capacity > 0.);
        debug_assert!(energy.is_finite());
        debug_assert!(energy >= 0.);
        debug_assert!(energy <= capacity);

        Self { capacity, energy }
    }

    /// The maximum capacity of the battery in joules.
    pub fn capacity(&self) -> f64 {
        self.capacity
    }

    /// The current energy level of the battery in joules.
    pub fn energy(&self) -> f64 {
        self.energy
    }

    /// Directly changes the energy level of the battery by the given amount of energy.
    fn change(&mut self, delta: f64) {
        debug_assert!(delta.is_finite());

        self.energy = (self.energy + delta).clamp(0., self.capacity);
    }
}

fn spawn_battery(mut commands: Commands, newly_spawned_units: Query<Entity, Added<Active>>) {
    for entity in newly_spawned_units.iter() {
        commands.entity(entity).insert(Battery::default());
    }
}

/// Discharges the batteries of all units.
///
/// # Arguments
///
/// * `time` - The time since the last update.
///
/// * `battery` - The battery.
pub(crate) fn discharge_battery(time: Res<Time>, mut battery: Query<&mut Battery>) {
    let delta = time.delta_seconds();
    let discharge_delta = DISCHARGE_RATE * delta as f64;
    for mut battery in battery.iter_mut() {
        let energy = battery.energy();
        if energy == 0. {
            continue;
        }

        battery.change(-discharge_delta);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::time::TimePlugin;

    use super::*;
    use crate::battery::{Battery, DEFAULT_CAPACITY, DISCHARGE_RATE};

    #[test]
    fn test_discharge() {
        // make new bevy app
        let mut app = App::new();
        // add entity and battery
        let entity = app
            .world
            .spawn((
                Battery::default(), // 100 kJ capacity, 100 kJ energy
            ))
            .id();

        app.add_plugins((BatteryPlugin, TimePlugin));

        // run the app for 1 second
        app.update();
        std::thread::sleep(std::time::Duration::from_secs(1));
        app.update();

        // check that the battery has discharged by at least 1*rate and 1.5*rate at most
        let battery = app.world.get::<Battery>(entity).unwrap();
        println!("battery: {:?}", battery);

        assert!(battery.energy() <= DEFAULT_CAPACITY - DISCHARGE_RATE);
        assert!(battery.energy() >= DEFAULT_CAPACITY - DISCHARGE_RATE * 1.5);
    }
}
