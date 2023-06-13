#![allow(unused_variables)]
//! This module implements final (i.e. parsed and validated) game configuration
//! objects and their building from persistent configuration.

use anyhow::{ensure, Context, Error, Result};
use async_std::path::Path;
use conf_macros::Config;
use de_uom::{LogicalPixel, Metre};
use serde::Deserialize;
use url::Url;

use crate::bundle_config;

// --------------------
// Config structs hold deserialized and validated data before
// further processing or packaging into Configuration

#[derive(Deserialize, Config, Debug, Clone)]
pub struct MultiplayerConf {
    #[ensure(server.scheme() == "http", "Only `http` scheme is allowed for `server`.")]
    pub server: Url,
}

#[derive(Deserialize, Config, Debug, Clone)]
pub struct Camera {
    pub scroll_inverted: bool,

    #[is_finite]
    #[ensure(*move_margin > 0., "`move_margin` must be positive.")]
    pub move_margin: f32,

    #[ensure(*min_distance >= 10., "`min_distance` must be larger or equal to 10.0.")]
    pub min_distance: f32,

    #[ensure(*max_distance <= 300., "`max_distance` must be smaller or equal to 300.0.")]
    #[ensure(*max_distance > *min_distance, "`max_distance` must be larger than `min_distance`.")]
    pub max_distance: f32,

    #[ensure(*wheel_zoom_sensitivity > 1., "`wheel_zoom_sensitivity` must be greater than 1.0.")]
    pub wheel_zoom_sensitivity: f32,

    #[ensure(*touchpad_zoom_sensitivity > 1., "`touchpad_zoom_sensitivity` must be greater than 1.0.")]
    pub touchpad_zoom_sensitivity: f32,

    #[ensure(*rotation_sensitivity > 0., "`rotation_sensitivity` must be greater than 0.0.")]
    pub rotation_sensitivity: f32,
}
// --------------------

// ---- default implementations ----

impl Default for MultiplayerConf {
    fn default() -> Self {
        Self {
            server: Url::parse("http://lobby.de_game.org").unwrap(),
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
            touchpad_zoom_sensitivity: 1.1,
            rotation_sensitivity: 0.01,
            scroll_inverted: false,
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
    pub fn server(&self) -> &Url {
        &self.server
    }
}

// Bundle configuration neatly into a single struct
bundle_config!(
    camera: CameraConf: Camera, // Conf file -> Camera -> CameraConf
    multiplayer: MultiplayerConf: MultiplayerConf  // Conf file -> MultiplayerConf
);
