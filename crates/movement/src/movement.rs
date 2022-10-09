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

/// Ideal velocity induced by a global path plan.
#[derive(Component, Default)]
pub(crate) struct DesiredMovement {
    velocity: Vec2,
}

impl DesiredMovement {
    pub(crate) fn velocity(&self) -> Vec2 {
        self.velocity
    }

    pub(crate) fn set_velocity(&mut self, velocity: Vec2) {
        self.velocity = velocity;
    }
}

fn setup_entities(
    mut commands: Commands,
    objects: Query<Entity, (With<MovableSolid>, Without<DesiredMovement>)>,
) {
    for entity in objects.iter() {
        commands.entity(entity).insert(DesiredMovement::default());
    }
}
