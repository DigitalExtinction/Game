use bevy::prelude::*;

pub(crate) mod component;
mod systems;

pub(crate) struct BatteryPlugin;

impl Plugin for BatteryPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(systems::discharge_battery);
    }
}

/// The rate at which the battery discharges in kiloJoules per second.
const DISCHARGE_RATE: f32 = 30.;
/// The minimum energy level in kiloJoules at which the unit can still move.
const MIN_MOVE_ENERGY: f32 = 74.; // based on how many a car uses per second
/// The minimum energy level in kiloJoules at which the unit can still attack.
const MIN_ATTACK_ENERGY: f32 = 20_000.; // google said that a rail-gun takes 25 Mj

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::time::TimePlugin;
    use crate::battery::{component, DISCHARGE_RATE};

    #[test]
    fn test_discharge() {
        // make new bevy app
        let mut app = App::new();
        // add entity and battery
        let entity = app.world.spawn((
            component::Battery::new(100_000., 100_000.), // 100 kJ capacity, 100 kJ energy
            )).id();

        // add the plugin
        app.add_plugin(super::BatteryPlugin);
        app.add_plugin(TimePlugin);

        // run the app for 1 second
        app.update();
        std::thread::sleep(std::time::Duration::from_secs(1));
        app.update();

        // check that the battery has discharged by at least 1*rate and 1.5*rate at most
        let battery = app.world.get::<component::Battery>(entity).unwrap();
        println!("battery: {:?}", battery);

        assert!(battery.energy() <= 100_000. - DISCHARGE_RATE);
        assert!(battery.energy() >= 100_000. - DISCHARGE_RATE * 1.5);
    }
}