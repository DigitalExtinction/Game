use bevy::prelude::*;
use de_core::state::GameState;
use iyes_loopless::prelude::*;

use super::draw::DrawingParam;

const TERRAIN_COLOR: Color = Color::rgb(0.61, 0.46, 0.32);

pub(super) struct FillPlugin;

impl Plugin for FillPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new().with_system(clear_system.run_in_state(GameState::Playing)),
        );
    }
}

fn clear_system(mut drawing: DrawingParam) {
    let mut drawing = drawing.drawing();
    drawing.fill(TERRAIN_COLOR);
}
