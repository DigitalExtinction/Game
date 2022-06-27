//! Object spawning and drafting functionalities.

use bevy::{app::PluginGroupBuilder, prelude::*};
pub use spawner::SpawnBundle;
use spawner::SpawnerPlugin;

mod spawner;

pub struct SpawnerPluginGroup;

impl PluginGroup for SpawnerPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(SpawnerPlugin);
    }
}
