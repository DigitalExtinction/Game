use bevy::prelude::*;

use crate::battery::component::Battery;
use crate::battery::{DISCHARGE_RATE};


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
        let discharge = DISCHARGE_RATE * delta;

        battery.try_discharge(discharge);
    }
}