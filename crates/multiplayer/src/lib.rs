use bevy::{app::PluginGroupBuilder, prelude::*};

pub use crate::config::{MultiplayerGameConfig, ServerPort};
use crate::{netstate::NetStatePlugin, network::NetworkPlugin};

mod config;
mod netstate;
mod network;

pub struct MultiplayerPluginGroup;

impl PluginGroup for MultiplayerPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(NetStatePlugin)
            .add(NetworkPlugin)
    }
}
