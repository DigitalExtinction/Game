use bevy::prelude::*;

pub(crate) struct BatteryPlugin;

impl Plugin for BatteryPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(discharge_battery);
    }
}

/// The rate at which the battery discharges in Joules per second.
const DISCHARGE_RATE: f64 = 30_000.;
/// The minimum energy level in kiloJoules at which the unit can still move.
const MIN_MOVE_ENERGY: EnergyUnit = EnergyUnit::Kilojoules(74.); // based on how many a car uses per second
/// The minimum energy level in kiloJoules at which the unit can still attack.
const MIN_ATTACK_ENERGY: EnergyUnit = EnergyUnit::Megajoules(20.); // google said that a rail-gun takes 25 Mj
/// The minimum energy level in kiloJoules at which factories can still produce units.
const MIN_FACTORY_ENERGY: EnergyUnit = EnergyUnit::Megajoules(10.);
/// The default capacity of the battery in kiloJoules.
const DEFAULT_CAPACITY: EnergyUnit = EnergyUnit::Megajoules(100.); // 100 Mj

pub enum EnergyUnit {
    Joules(f64),
    Kilojoules(f64),
    Megajoules(f64),
    Gigajoules(f64),
}

impl EnergyUnit {
    pub fn to_joules(&self) -> f64 {
        match self {
            Self::Joules(j) => *j,
            Self::Kilojoules(kj) => kj * 1000.,
            Self::Megajoules(mj) => mj * 1_000_000.,
            Self::Gigajoules(gj) => gj * 1_000_000_000.,
        }
    }

    pub fn to_kilojoules(&self) -> f64 {
        match self {
            Self::Joules(j) => j / 1000.,
            Self::Kilojoules(kj) => *kj,
            Self::Megajoules(mj) => mj * 1000.,
            Self::Gigajoules(gj) => gj * 1_000_000.,
        }
    }
}

/// The battery component is used to store the energy level of an entity.
///
/// # fields
/// * `capacity` - The maximum capacity of the battery in joules.
/// * `energy` - The current energy level of the battery in joules.
#[derive(Component, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Battery {
    capacity: f64,
    energy: f64,
}

impl Default for Battery {
    fn default() -> Self {
        Self {
            capacity: DEFAULT_CAPACITY.to_joules(), // convert to joules
            energy: DEFAULT_CAPACITY.to_joules(),   // convert to joules
        }
    }
}



impl Battery {
    pub fn new(capacity: EnergyUnit, energy: EnergyUnit) -> Self {
        let capacity = capacity.to_joules();
        let energy = energy.to_joules();
        debug_assert!(capacity.is_finite());
        debug_assert!(capacity > 0.);
        debug_assert!(energy.is_finite());
        debug_assert!(energy >= 0.);
        debug_assert!(energy <= capacity);

        Self { capacity, energy }
    }

    /// The maximum capacity of the battery in kilojoules.
    pub fn capacity(&self) -> f64 {
        self.capacity
    }

    /// The current energy level of the battery in kilojoules.
    pub fn energy(&self) -> f64 {
        self.energy
    }

    /// Does the battery contain enough energy to move?
    pub fn can_move(&self) -> bool {
        self.energy >= MIN_MOVE_ENERGY .to_joules()
    }

    /// Does the battery contain enough energy to attack?
    pub fn can_fire(&self) -> bool {
        self.energy >= MIN_ATTACK_ENERGY.to_joules()
    }

    /// Does the battery contain enough energy to produce units?
    pub fn can_produce(&self) -> bool {
        self.energy >= MIN_FACTORY_ENERGY.to_joules()
    }

    /// Directly sets the energy level of the battery.
    pub fn set_energy(&mut self, energy: f64) {
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
    pub fn try_discharge(&mut self, energy: EnergyUnit) -> bool {
        let energy = energy.to_joules();
        debug_assert!(energy.is_finite());
        debug_assert!(energy >= 0.);

        if energy >= self.energy {
            return false;
        }

        self.change(-energy);
        true
    }

    /// Directly changes the energy level of the battery by the given amount of energy.
    pub fn change(&mut self, delta: f64) {
        debug_assert!(delta.is_finite());

        self.energy = (self.energy + delta).clamp(0., self.capacity);
    }

    /// Get fraction of energy remaining in the battery. (a number between 0 and 1)
    pub fn energy_percentage(&self) -> f64 {
        self.energy / self.capacity
    }
}

/// Discharges the battery of a unit.
///
/// # Parameters
///
/// - `time`: The time since the last update in seconds.
/// - `battery`: The battery.
pub(crate) fn discharge_battery(time: Res<Time>, mut battery: Query<&mut Battery>) {
    for mut battery in battery.iter_mut() {
        let energy = battery.energy();
        if energy == 0. {
            continue;
        }
        let delta = time.delta_seconds();
        let discharge = (DISCHARGE_RATE) * delta as f64;

        battery.try_discharge(EnergyUnit::Joules(discharge));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::time::TimePlugin;

    use crate::battery::{Battery, DISCHARGE_RATE};

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

        // add the plugin
        app.add_plugin(super::BatteryPlugin);
        app.add_plugin(TimePlugin);

        // run the app for 1 second
        app.update();
        std::thread::sleep(std::time::Duration::from_secs(1));
        app.update();

        // check that the battery has discharged by at least 1*rate and 1.5*rate at most
        let battery = app.world.get::<Battery>(entity).unwrap();
        println!("battery: {:?}", battery);

        assert!(battery.energy() <= 100_000. * 1000. - (DISCHARGE_RATE));
        assert!(battery.energy() >= 100_000. * 1000. - (DISCHARGE_RATE) * 1.5);
    }
}
