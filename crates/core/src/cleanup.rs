use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::state::AppState;

pub(crate) struct CleanupPlugin;

impl Plugin for CleanupPlugin {
    fn build(&self, app: &mut App) {
        app.add_exit_system(AppState::InGame, cleanup);
    }
}

/// Mark all entities which should be recursively despawned after the game is
/// exited with this component.
#[derive(Component)]
pub struct DespawnOnGameExit;

fn cleanup(mut commands: Commands, query: Query<Entity, With<DespawnOnGameExit>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
