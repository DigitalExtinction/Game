use bevy::prelude::*;
use de_core::{
    gconfig::GameConfig,
    objects::{ActiveObjectType, ObjectType, PLAYER_MAX_BUILDINGS, PLAYER_MAX_UNITS},
    player::Player,
    stages::GameStage,
};

pub(crate) struct CounterPlugin;

impl Plugin for CounterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObjectCounter>()
            .add_system_to_stage(GameStage::PostUpdate, recount);
    }
}

/// Current count of buildings and units belonging to local player.
#[derive(Default)]
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
