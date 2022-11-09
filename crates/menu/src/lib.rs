use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use mainmenu::MainMenuPlugin;
use menu::MenuPlugin;

mod mainmenu;
mod menu;

pub struct MenuPluginGroup;

impl PluginGroup for MenuPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(MenuPlugin).add(MainMenuPlugin);
    }
}
