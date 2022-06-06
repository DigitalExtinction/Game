use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
pub use spawner::SpawnEvent;
use spawner::SpawnerPlugin;

mod spawner;

pub struct ObjectsPluginGroup;

impl PluginGroup for ObjectsPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(SpawnerPlugin);
    }
}
