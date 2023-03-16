use bevy::prelude::*;

mod actionbar;
mod details;
mod interaction;
mod menu;
mod minimap;
mod selection;

pub(crate) use interaction::HudNodes;
pub(crate) use menu::{GameMenuSet, ToggleGameMenu};
pub(crate) use selection::UpdateSelectionBoxEvent;

use self::{
    actionbar::ActionBarPlugin, details::DetailsPlugin, menu::MenuPlugin, minimap::MinimapPlugin,
    selection::SelectionPlugin,
};

const HUD_COLOR: Color = Color::BLACK;

pub(crate) struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SelectionPlugin)
            .add_plugin(DetailsPlugin)
            .add_plugin(ActionBarPlugin)
            .add_plugin(MenuPlugin)
            .add_plugin(MinimapPlugin);
    }
}
