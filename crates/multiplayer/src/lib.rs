//! This crate library implements multiplayer functionality for Digital
//! Extinction via [`MultiplayerPluginGroup`].
//!
//! Before a multiplayer game starts, networking and other systems must be
//! started. To do this send [`StartMultiplayerEvent`].
//!
//! After a multiplayer game ends, the multiplayer functionality should be shut
//! down via [`ShutdownMultiplayerEvent`].

use bevy::{app::PluginGroupBuilder, prelude::*};
use game::GamePlugin;
use lifecycle::LifecyclePlugin;
use messages::MessagesPlugin;
use playermsg::PlayerMsgPlugin;
use stats::StatsPlugin;

pub use crate::{
    config::{ConnectionType, NetGameConf},
    game::{
        GameJoinedEvent, GameOpenedEvent, GameReadinessEvent, PeerJoinedEvent, PeerLeftEvent,
        SetReadinessEvent,
    },
    lifecycle::{MultiplayerShuttingDownEvent, ShutdownMultiplayerEvent, StartMultiplayerEvent},
    messages::{MessagesSet, ToPlayersEvent},
    netstate::NetState,
    playermsg::{GameNetSet, NetEntities, NetRecvDespawnActiveEvent, NetRecvSpawnActiveEvent},
};
use crate::{netstate::NetStatePlugin, network::NetworkPlugin};

mod config;
mod game;
mod lifecycle;
mod messages;
mod netstate;
mod network;
mod playermsg;
mod stats;

pub struct MultiplayerPluginGroup;

impl PluginGroup for MultiplayerPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(NetStatePlugin)
            .add(LifecyclePlugin)
            .add(NetworkPlugin)
            .add(MessagesPlugin)
            .add(GamePlugin)
            .add(StatsPlugin)
            .add(PlayerMsgPlugin)
    }
}
