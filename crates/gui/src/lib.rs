//! This crate implements a plugin group and various events and system
//! parameters used as building blocks for 2D in-game and menu UI across the
//! game.

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
pub use button::ButtonCommands;
use button::ButtonPlugin;
pub use commands::GuiCommands;
pub use style::OuterStyle;
use text::TextPlugin;

mod button;
mod commands;
mod style;
mod text;

pub struct GuiPluginGroup;

impl PluginGroup for GuiPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(TextPlugin)
            .add(ButtonPlugin)
    }
}
