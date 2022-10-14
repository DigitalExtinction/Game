mod collider;
mod plugin;
mod terrain;

use bevy::{app::PluginGroupBuilder, prelude::*};
pub use collider::TerrainCollider;
use plugin::TerrainPlugin;
pub use terrain::TerrainBundle;

pub struct TerrainPluginGroup;

impl PluginGroup for TerrainPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(TerrainPlugin);
    }
}
