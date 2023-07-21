use std::ops::{Add, AddAssign};

use ahash::AHashMap;
use bevy::prelude::*;
use de_core::{
    gconfig::GameConfig,
    objects::ActiveObjectType,
    player::{Player, PlayerRange},
    state::AppState,
};

pub(crate) struct CounterPlugin;

impl Plugin for CounterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(OnExit(AppState::InGame), cleanup);
    }
}

#[derive(Resource)]
pub struct ObjectCounter {
    players: AHashMap<Player, PlayerObjectCounter>,
}

impl ObjectCounter {
    fn new(players: PlayerRange) -> Self {
        let mut map = AHashMap::with_capacity(players.len());
        for player in players {
            map.insert(player, PlayerObjectCounter::default());
        }
        Self { players: map }
    }

    pub fn player(&self, player: Player) -> Option<&PlayerObjectCounter> {
        self.players.get(&player)
    }

    pub(crate) fn player_mut(&mut self, player: Player) -> Option<&mut PlayerObjectCounter> {
        self.players.get_mut(&player)
    }
}

/// Current count of buildings and units belonging to a player.
#[derive(Default)]
pub struct PlayerObjectCounter {
    building_count: Count,
    unit_count: Count,
}

impl PlayerObjectCounter {
    pub fn total(&self) -> u32 {
        (self.building_count + self.unit_count).0
    }

    pub fn building_count(&self) -> u32 {
        self.building_count.0
    }

    pub fn unit_count(&self) -> u32 {
        self.unit_count.0
    }

    /// Updates number of objects by a given amount.
    ///
    /// # Panics
    ///
    /// Panics if the number of tracked objects goes below 0 or above 2^32 - 1;
    pub(crate) fn update(&mut self, object_type: ActiveObjectType, change: i32) {
        match object_type {
            ActiveObjectType::Building(_) => self.building_count += change,
            ActiveObjectType::Unit(_) => self.unit_count += change,
        }
    }
}

#[derive(Default, Clone, Copy)]
struct Count(u32);

impl Add for Count {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0.checked_add(other.0).unwrap())
    }
}

impl AddAssign<i32> for Count {
    fn add_assign(&mut self, other: i32) {
        if other >= 0 {
            self.0 = self.0.checked_add(other as u32).unwrap();
        } else {
            self.0 = self.0.checked_sub((-other) as u32).unwrap();
        }
    }
}

fn setup(mut commands: Commands, config: Res<GameConfig>) {
    commands.insert_resource(ObjectCounter::new(config.players()));
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<ObjectCounter>();
}
