use bevy::prelude::*;

mod interaction;
mod menu;
mod minimap;
mod panel;
mod selection;

pub(crate) use interaction::HudNodes;
pub(crate) use menu::{GameMenuSet, ToggleGameMenu};
pub(crate) use selection::UpdateSelectionBoxEvent;

use self::{
    menu::MenuPlugin, minimap::MinimapPlugin, panel::PanelPlugin, selection::SelectionPlugin,
};

const HUD_COLOR: Color = Color::BLACK;

pub(crate) struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SelectionPlugin)
            .add_plugin(PanelPlugin)
            .add_plugin(MenuPlugin)
            .add_plugin(MinimapPlugin);
    }
}
