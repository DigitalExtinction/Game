use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use cache::CachePlugin;
pub use spawner::SpawnEvent;
use spawner::SpawnerPlugin;

mod cache;
mod spawner;

pub struct ObjectsPluginGroup;

impl PluginGroup for ObjectsPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(CachePlugin).add(SpawnerPlugin);
    }
}
