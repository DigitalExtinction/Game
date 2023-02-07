use bevy::prelude::*;
use de_core::{
    gconfig::GameConfig,
    objects::{ActiveObjectType, ObjectType, PLAYER_MAX_BUILDINGS, PLAYER_MAX_UNITS},
    player::Player,
    stages::GameStage,
    state::AppState,
};
use iyes_loopless::prelude::*;

pub(crate) struct CounterPlugin;

impl Plugin for CounterPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::InGame, setup)
            .add_exit_system(AppState::InGame, cleanup)
            .add_system_to_stage(
                GameStage::PostUpdate,
                recount.run_in_state(AppState::InGame),
            );
    }
}

/// Current count of buildings and units belonging to local player.
#[derive(Default, Resource)]
pub struct ObjectCounter {
    building_count: usize,
    unit_count: usize,
}

impl ObjectCounter {
    pub fn building_count(&self) -> usize {
        self.building_count
    }

    pub fn unit_count(&self) -> usize {
        self.unit_count
    }
}

fn setup(mut commands: Commands) {
    commands.init_resource::<ObjectCounter>();
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<ObjectCounter>();
}

fn recount(
    config: Res<GameConfig>,
    mut counter: ResMut<ObjectCounter>,
    objects: Query<(&Player, &ObjectType)>,
) {
    counter.building_count = 0;
    counter.unit_count = 0;

    for (&player, &object_type) in objects.iter() {
        if let ObjectType::Active(object_type) = object_type {
            if config.is_local_player(player) {
                match object_type {
                    ActiveObjectType::Building(_) => counter.building_count += 1,
                    ActiveObjectType::Unit(_) => counter.unit_count += 1,
                }
            }
        }
    }

    if counter.building_count > PLAYER_MAX_BUILDINGS {
        panic!("Maximum number of buildings surpassed.");
    }
    if counter.unit_count > PLAYER_MAX_UNITS {
        panic!("Maximum number of units surpassed.");
    }
}
