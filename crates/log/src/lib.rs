mod setup;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

use crate::setup::LogPlugin;

pub struct LogPluginGroup;

impl PluginGroup for LogPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(LogPlugin)
    }
}
