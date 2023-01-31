use bevy::prelude::*;

mod interaction;
mod menu;
mod panel;
mod selection;

pub(crate) use interaction::HudNodes;
pub(crate) use menu::{GameMenuLabel, ToggleGameMenu};
pub(crate) use selection::UpdateSelectionBoxEvent;

use self::{menu::MenuPlugin, panel::PanelPlugin, selection::SelectionPlugin};

pub(crate) struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SelectionPlugin)
            .add_plugin(PanelPlugin)
            .add_plugin(MenuPlugin);
    }
}
