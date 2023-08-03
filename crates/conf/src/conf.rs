#![allow(unused_variables)]
//! This module implements final (i.e. parsed and validated) game configuration
//! objects and their building from persistent configuration.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::{ensure, Context, Error, Result};
use async_std::path::Path;
use conf_macros::Config;
use de_uom::{LogicalPixel, Metre};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::bundle_config;

// --------------------
// Config structs hold deserialized and validated data before
// further processing or packaging into Configuration

#[derive(Deserialize, Serialize, Config, Debug, Clone)]
pub struct MultiplayerConf {
    #[ensure(lobby.scheme() == "http", "Only `http` scheme is allowed for `lobby`.")]
    lobby: Url,
    connector: SocketAddr,
}

#[derive(Deserialize, Serialize, Config, Debug, Clone)]
pub struct Camera {
    scroll_inverted: bool,

    #[is_finite]
    #[ensure(*move_margin > 0., "`move_margin` must be positive.")]
    move_margin: f32,

    #[ensure(*min_distance >= 10., "`min_distance` must be larger or equal to 10.0.")]
    min_distance: f32,

    #[ensure(*max_distance <= 300., "`max_distance` must be smaller or equal to 300.0.")]
    #[ensure(*max_distance > *min_distance, "`max_distance` must be larger than `min_distance`.")]
    max_distance: f32,

    #[ensure(*wheel_zoom_sensitivity > 1., "`wheel_zoom_sensitivity` must be greater than 1.0.")]
    wheel_zoom_sensitivity: f32,

    #[ensure(*touchpad_zoom_sensitivity > 1., "`touchpad_zoom_sensitivity` must be greater than 1.0.")]
    touchpad_zoom_sensitivity: f32,

    #[ensure(*rotation_sensitivity > 0., "`rotation_sensitivity` must be greater than 0.0.")]
    rotation_sensitivity: f32,
}

#[derive(Deserialize, Serialize, Config, Debug, Clone)]
pub struct AudioConf {
    #[is_finite]
    #[ensure(*music_volume >= 0., "`master_volume` must be greater than or equal to 0.0.")]
    #[ensure(*music_volume <= 1., "`master_volume` must be smaller or equal to 1.0.")]
    master_volume: f32,

    #[is_finite]
    #[ensure(*music_volume >= 0., "`sound_volume` must be greater than or equal to 0.0.")]
    #[ensure(*music_volume <= 1., "`sound_volume` must be smaller or equal to 1.0.")]
    sound_volume: f32,

    #[is_finite]
    #[ensure(*music_volume >= 0., "`music_volume` must be greater than or equal to 0.0.")]
    #[ensure(*music_volume <= 1., "`music_volume` must be smaller or equal to 1.0.")]
    music_volume: f32,
}
// --------------------

// ---- default implementations ----

impl Default for MultiplayerConf {
    fn default() -> Self {
        Self {
            lobby: Url::parse("http://lobby.de_game.org").unwrap(),
            connector: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8082),
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            move_margin: 40.,
            min_distance: 20.,
            max_distance: 80.,
            wheel_zoom_sensitivity: 1.1,
            touchpad_zoom_sensitivity: 1.01,
            rotation_sensitivity: 0.008,
            scroll_inverted: false,
        }
    }
}

impl Default for AudioConf {
    fn default() -> Self {
        Self {
            master_volume: 1.,
            sound_volume: 1.,
            music_volume: 1.,
        }
    }
}

// --------------------

// for this more complicated data structure, we need to
// implement TryInto so that the macro can convert it
// into its desired data structure before packing into
// Configuration
impl TryInto<CameraConf> for Camera {
    type Error = Error;

    fn try_into(self) -> Result<CameraConf> {
        Ok(CameraConf {
            move_margin: LogicalPixel::new(self.move_margin),
            min_distance: Metre::new(self.min_distance),
            max_distance: Metre::new(self.max_distance),
            wheel_zoom_sensitivity: self.wheel_zoom_sensitivity,
            touchpad_zoom_sensitivity: self.touchpad_zoom_sensitivity,
            rotation_sensitivity: self.rotation_sensitivity,
            scroll_inverted: self.scroll_inverted,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CameraConf {
    move_margin: LogicalPixel,
    min_distance: Metre,
    max_distance: Metre,
    wheel_zoom_sensitivity: f32,
    touchpad_zoom_sensitivity: f32,
    rotation_sensitivity: f32,
    scroll_inverted: bool,
}

// ---- config impls ----

impl CameraConf {
    /// Horizontal camera movement is initiated if mouse cursor is within this
    /// distance to window edge.
    pub fn move_margin(&self) -> LogicalPixel {
        self.move_margin
    }

    /// Minimum camera distance from terrain achievable with zooming along.
    pub fn min_distance(&self) -> Metre {
        self.min_distance
    }

    /// Maximum camera distance from terrain achievable with zooming alone.
    pub fn max_distance(&self) -> Metre {
        self.max_distance
    }

    /// Scale factor (i.e `distance * factor`) applied after single mouse wheel
    /// tick.
    pub fn wheel_zoom_sensitivity(&self) -> f32 {
        self.wheel_zoom_sensitivity
    }

    /// Scale factor (i.e. `distance * drag_size * factor`) applied after
    /// sliding on touch pad.
    pub fn touchpad_zoom_sensitivity(&self) -> f32 {
        self.touchpad_zoom_sensitivity
    }

    /// Mouse drag by `d` logical pixels will lead to rotation by `d *
    /// rotation_sensitivity` radians.
    pub fn rotation_sensitivity(&self) -> f32 {
        self.rotation_sensitivity
    }

    /// Whether scrolling should be inverted.
    pub fn scroll_inverted(&self) -> bool {
        self.scroll_inverted
    }
}

impl MultiplayerConf {
    /// Server URL for lobby connections.
    pub fn lobby(&self) -> &Url {
        &self.lobby
    }

    /// Socket address of a DE Connector main sever.
    pub fn connector(&self) -> SocketAddr {
        self.connector
    }
}

impl AudioConf {
    /// Whether audio is enabled (master volume is above zero).
    pub fn audio_enabled(&self) -> bool {
        self.master_volume > 0.
    }

    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Whether SFX are enabled (sound and master volume are above zero).
    pub fn sound_enabled(&self) -> bool {
        self.audio_enabled() && self.sound_volume > 0.
    }

    pub fn sound_volume(&self) -> f32 {
        self.master_volume * self.sound_volume
    }

    /// Whether music is enabled (music and master volume are above zero).
    pub fn music_enabled(&self) -> bool {
        self.audio_enabled() && self.music_volume > 0.
    }

    pub fn music_volume(&self) -> f32 {
        self.master_volume * self.music_volume
    }
}

// Bundle configuration neatly into a single struct
bundle_config!(
    camera: CameraConf: Camera, // Conf file -> Camera -> CameraConf
    multiplayer: MultiplayerConf: MultiplayerConf,  // Conf file -> MultiplayerConf
    audio: AudioConf: AudioConf
);
