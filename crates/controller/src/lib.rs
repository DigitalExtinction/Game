//! This crate implements handling of user input.

use areaselect::AreaSelectPlugin;
use bevy::{app::PluginGroupBuilder, prelude::*};
use command::CommandPlugin;
use draft::DraftPlugin;
use dragselect::DragSelectPlugin;
use hud::HudPlugin;
use mouse::MousePlugin;
use selection::SelectionPlugin;

mod areaselect;
mod command;
mod draft;
mod dragselect;
mod frustum;
mod hud;
mod keyboard;
mod mouse;
mod selection;

const SELECTION_BAR_ID: u32 = 0;
const POINTER_BAR_ID: u32 = 1;

pub struct ControllerPluginGroup;

impl PluginGroup for ControllerPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(DragSelectPlugin)
            .add(AreaSelectPlugin)
            .add(MousePlugin)
            .add(CommandPlugin)
            .add(SelectionPlugin)
            .add(DraftPlugin)
            .add(HudPlugin)
    }
}
