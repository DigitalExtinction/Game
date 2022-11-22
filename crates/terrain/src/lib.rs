mod collider;
mod marker;
mod plugin;
mod shader;
mod terrain;

use bevy::{app::PluginGroupBuilder, prelude::*};
pub use collider::TerrainCollider;
pub use marker::CircleMarker;
use marker::MarkerPlugin;
use plugin::TerrainPlugin;
pub use terrain::TerrainBundle;

pub struct TerrainPluginGroup;

impl PluginGroup for TerrainPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(TerrainPlugin)
            .add(MarkerPlugin)
    }
}
