use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

use crate::music::MusicPlugin;

mod music;

pub struct AudioPluginGroup;

impl PluginGroup for AudioPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(MusicPlugin)
    }
}
