//! This crate implements networking in Digital Extinction.

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use io::IoPlugin;

mod network;
mod io;

pub struct NetPluginGroup;

impl PluginGroup for NetPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(IoPlugin)
    }
}
