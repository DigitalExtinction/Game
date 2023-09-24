use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

use crate::music::MusicPlugin;
use crate::spatial::SpatialSoundPlugin;

mod music;
pub mod spatial;

pub struct AudioPluginGroup;

impl PluginGroup for AudioPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(MusicPlugin)
            .add(SpatialSoundPlugin)
    }
}
