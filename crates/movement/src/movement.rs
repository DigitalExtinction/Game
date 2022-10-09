use bevy::prelude::*;
use de_core::{objects::MovableSolid, stages::GameStage, state::GameState};
use iyes_loopless::prelude::*;

pub(crate) struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            GameStage::PreMovement,
            setup_entities.run_in_state(GameState::Playing),
        );
    }
}

#[derive(Component, Default)]
pub(crate) struct Movement {
    /// Ideal velocity induced by a global path plan.
    desired: Vec3,
}

impl Movement {
    pub(crate) fn desired_velocity(&self) -> Vec3 {
        self.desired
    }

    pub(crate) fn stop(&mut self) {
        self.desired = Vec3::ZERO;
    }

    pub(crate) fn set_desired_velocity(&mut self, velocity: Vec3) {
        self.desired = velocity;
    }
}

fn setup_entities(
    mut commands: Commands,
    objects: Query<Entity, (With<MovableSolid>, Without<Movement>)>,
) {
    for entity in objects.iter() {
        commands.entity(entity).insert(Movement::default());
    }
}
