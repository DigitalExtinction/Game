use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use mainmenu::MainMenuPlugin;
use mapselection::MapSelectionPlugin;
use menu::MenuPlugin;

mod mainmenu;
mod mapselection;
mod menu;

pub struct MenuPluginGroup;

impl PluginGroup for MenuPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(MenuPlugin)
            .add(MainMenuPlugin)
            .add(MapSelectionPlugin);
    }
}
