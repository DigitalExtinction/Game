mod plugin;

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use plugin::UiPlugin;
pub use plugin::UpdateSelectionBoxEvent;

pub struct UiPluginGroup;

impl PluginGroup for UiPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(UiPlugin)
    }
}
