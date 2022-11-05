use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use mainmenu::MainMenuPlugin;

mod mainmenu;

pub struct MenuPluginGroup;

impl PluginGroup for MenuPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(MainMenuPlugin);
    }
}
