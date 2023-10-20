//! This module implements point based spatial indexing and various geometry
//! based look ups (e.g. range query).

use ahash::AHashMap;
use bevy::prelude::*;
use de_core::{
    gconfig::GameConfig, player::PlayerComponent, schedule::PostMovement, state::AppState,
};
use de_types::{player::Player, projection::ToFlat};

use self::tree::Tree;

mod grid;
mod subdivision;
mod tree;

pub(super) struct PointIndexPlugin;

impl Plugin for PointIndexPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(OnExit(AppState::InGame), cleanup)
            .add_systems(PostUpdate, spawned.run_if(in_state(AppState::InGame)))
            .add_systems(PostMovement, update.run_if(in_state(AppState::InGame)));
    }
}

// TODO docs
#[derive(Resource)]
pub struct PlayerPointIndex {
    players: AHashMap<Player, Tree>,
}

impl PlayerPointIndex {
    fn new() -> Self {
        Self {
            players: AHashMap::new(),
        }
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(PlayerPointIndex::new());
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<PlayerPointIndex>();
}

fn spawned(
    mut index: ResMut<PlayerPointIndex>,
    spawned: Query<(Entity, &Transform, &PlayerComponent), Added<PlayerComponent>>,
) {
    for (entity, transform, player) in spawned.iter() {
        index
            .players
            .entry(**player)
            .or_insert_with(Tree::new)
            .add(entity, transform.translation.to_flat());
    }
}

fn update(
    mut index: ResMut<PlayerPointIndex>,
    updated: Query<(Entity, &Transform, &PlayerComponent), Changed<Transform>>,
) {
    for (entity, transform, player) in updated.iter() {
        index
            .players
            .get_mut(player)
            .unwrap()
            .update(entity, transform.translation.to_flat());
    }
}

// TODO treat despawned entities
// fn despawned(mut index: ResMut<PlayerPointIndex>, spawned: RemovedComponents<PlayerComponent>) {
//     for (entity, transform, player) in spawned.iter() {
//         index
//             .players
//             .entry(**player)
//             .or_insert_with(Tree::new)
//             .add(entity, transform.translation.to_flat());
//     }
// }
