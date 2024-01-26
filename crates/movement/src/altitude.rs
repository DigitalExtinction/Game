use bevy::prelude::*;
use de_core::{
    gamestate::GameState,
    objects::{MovableSolid, ObjectTypeComponent},
    schedule::{Movement, PreMovement},
    state::AppState,
};
use de_objects::SolidObjects;

use crate::{
    movement::DesiredVelocity,
    repulsion::{RepulsionLables, RepulsionVelocity},
    G_ACCELERATION, MAX_V_ACCELERATION, MAX_V_SPEED,
};

pub(crate) struct AltitudePlugin;

impl Plugin for AltitudePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreMovement,
            setup_entities.run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Movement,
            update
                .run_if(in_state(GameState::Playing))
                .in_set(AltitudeSet::Update)
                .after(RepulsionLables::Apply),
        );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub(crate) enum AltitudeSet {
    Update,
}

#[derive(Component, Default)]
pub(crate) struct DesiredClimbing(f32);

impl DesiredClimbing {
    pub(crate) fn speed(&self) -> f32 {
        self.0
    }

    pub(crate) fn set_speed(&mut self, speed: f32) {
        self.0 = speed;
    }
}

fn setup_entities(
    mut commands: Commands,
    objects: Query<Entity, (With<MovableSolid>, Without<DesiredClimbing>)>,
) {
    for entity in objects.iter() {
        commands.entity(entity).insert(DesiredClimbing::default());
    }
}

fn update(
    solids: SolidObjects,
    mut objects: Query<(
        &ObjectTypeComponent,
        &mut DesiredVelocity<RepulsionVelocity>,
        &mut DesiredClimbing,
        &Transform,
    )>,
) {
    objects
        .par_iter_mut()
        .for_each(|(object_type, mut horizontal, mut climbing, transform)| {
            let Some(flight) = solids.get(**object_type).flight() else {
                return;
            };
            let height = transform.translation.y;

            let desired_height = if horizontal.stationary() {
                0.
            } else {
                if height < flight.min_height() {
                    horizontal.stop();
                }
                flight.max_height()
            };

            let remaining = desired_height - height;
            let max_acceleration = if remaining > 0. {
                G_ACCELERATION
            } else {
                MAX_V_ACCELERATION
            };
            // Make sure that the object can slow down soon enough.
            let desired = remaining.signum()
                * MAX_V_SPEED.min((2. * remaining.abs() * max_acceleration).sqrt());

            // Avoid change detection when possible.
            if climbing.speed() != desired {
                climbing.set_speed(desired);
            }
        });
}
