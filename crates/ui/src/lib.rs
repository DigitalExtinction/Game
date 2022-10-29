mod plugin;

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use plugin::UiPlugin;
pub use plugin::UpdateSelectionBoxEvent;

pub struct UiPluginGroup;

impl PluginGroup for UiPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(UiPlugin);
    }
}
