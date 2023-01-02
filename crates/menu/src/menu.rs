use bevy::prelude::*;
use de_core::state::AppState;
use iyes_loopless::prelude::*;

pub(crate) struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::InMenu, setup)
            .add_exit_system(AppState::InMenu, cleanup);
    }
}

/// This system recursively despawns all `Node`s with no parents.
pub(crate) fn despawn_root_nodes(
    mut commands: Commands,
    nodes: Query<Entity, (With<Node>, Without<Parent>)>,
) {
    for entity in nodes.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn cleanup(mut commands: Commands, camera: Query<Entity, With<Camera2d>>) {
    for entity in camera.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
