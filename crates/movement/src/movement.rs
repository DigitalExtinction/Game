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
#[derive(Component)]
pub(crate) struct DesiredMovement {
    velocity: Vec2,
    stopped: bool,
}

impl DesiredMovement {
    pub(crate) fn velocity(&self) -> Vec2 {
        self.velocity
    }

    pub(crate) fn stopped(&self) -> bool {
        self.stopped
    }

    pub(crate) fn stop(&mut self) {
        self.velocity = Vec2::ZERO;
        self.stopped = true;
    }

    pub(crate) fn start(&mut self, velocity: Vec2) {
        self.velocity = velocity;
        self.stopped = false;
    }

    pub(crate) fn update(&mut self, velocity: Vec2) {
        self.velocity = velocity;
    }
}

impl Default for DesiredMovement {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            stopped: true,
        }
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
