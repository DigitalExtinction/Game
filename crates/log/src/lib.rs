mod setup;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

use crate::setup::LogPlugin;

pub struct LogPluginGroup {
    pub current_log_file: String,
}

impl PluginGroup for LogPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(LogPlugin {
            current_log_file: self.current_log_file,
        })
    }
}
