//! This crate implements handling of user input.

use bevy::{app::PluginGroupBuilder, prelude::*};
use commands::CommandsPlugin;
use draft::DraftPlugin;
use hud::HudPlugin;
use mouse::MousePlugin;
use selection::SelectionPlugin;

mod commands;
mod draft;
mod frustum;
mod hud;
mod mouse;
mod selection;

const SELECTION_BAR_ID: u32 = 0;
const POINTER_BAR_ID: u32 = 1;

pub struct ControllerPluginGroup;

impl PluginGroup for ControllerPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(MousePlugin)
            .add(CommandsPlugin)
            .add(SelectionPlugin)
            .add(DraftPlugin)
            .add(HudPlugin)
    }
}
