//! This crate implements networking in Digital Extinction.

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use io::IoPlugin;

mod accounting;
mod buffer;
mod io;
mod msg;
mod net;

pub struct NetPluginGroup;

impl PluginGroup for NetPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(IoPlugin)
    }
}
