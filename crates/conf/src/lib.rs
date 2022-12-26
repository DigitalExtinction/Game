//! This crate implements functionality around game configuration:
//!
//! * Automatic (re)loading of the configuration from a YAML file (during
//!   MenuState::Loading state).
//!
//! * Parsing, validation and configuration provisioning.

mod conf;
mod io;
mod persisted;
mod plugin;

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
pub use conf::*;
use plugin::ConfPlugin;

pub struct ConfigPluginGroup;

impl PluginGroup for ConfigPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(ConfPlugin)
    }
}
