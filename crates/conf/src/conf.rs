//! This module implements final (i.e. parsed and validated) game configuration
//! objects and their building from persistent configuration objects.

use anyhow::{bail, ensure, Context, Error, Result};
use bevy::prelude::Resource;
use de_uom::{InverseLogicalPixel, LogicalPixel, Metre};
use url::Url;

use crate::persisted::{self, *};

/// Top-level game configuration object.
#[derive(Resource)]
pub struct Configuration {
    multiplayer: MultiplayerConf,
    camera: CameraConf,
}

impl Configuration {
    pub fn multiplayer(&self) -> &MultiplayerConf {
        &self.multiplayer
    }

    pub fn camera(&self) -> &CameraConf {
        &self.camera
    }
}

impl TryFrom<persisted::Configuration> for Configuration {
    type Error = Error;

    fn try_from(conf: persisted::Configuration) -> Result<Self> {
        Ok(Self {
            multiplayer: conf
                .multiplayer
                .try_into()
                .context("`multiplayer` validation failed")?,
            camera: conf
                .camera
                .try_into()
                .context("`camera` validation failed")?,
        })
    }
}

pub struct MultiplayerConf {
    server: Url,
}

impl MultiplayerConf {
    pub fn server(&self) -> &Url {
        &self.server
    }
}

impl TryFrom<Option<Multiplayer>> for MultiplayerConf {
    type Error = Error;

    fn try_from(conf: Option<Multiplayer>) -> Result<Self> {
        let server = conf
            .and_then(|c| c.server)
            .unwrap_or_else(|| Url::parse("http://lobby.de-game.org").unwrap());
        if server.scheme() != "http" {
            bail!("Only `http` scheme is allowed for `server`.")
        }

        Ok(Self { server })
    }
}

pub struct CameraConf {
    move_margin: LogicalPixel,
    min_distance: Metre,
    max_distance: Metre,
    wheel_zoom_sensitivity: f32,
    touchpad_zoom_sensitivity: f32,
    rotation_sensitivity: InverseLogicalPixel,
}

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
    pub fn rotation_sensitivity(&self) -> InverseLogicalPixel {
        self.rotation_sensitivity
    }
}

impl TryFrom<Option<Camera>> for CameraConf {
    type Error = Error;

    fn try_from(conf: Option<Camera>) -> Result<Self> {
        let conf = conf.as_ref();

        let move_margin = conf.and_then(|c| c.move_margin).unwrap_or(40.);
        ensure!(move_margin.is_finite(), "`move_margin` must be finite.");
        ensure!(move_margin > 0., "`move_margin` must be positive.");

        let min_distance = conf.and_then(|c| c.min_distance).unwrap_or(20.);
        ensure!(min_distance.is_finite(), "`min_distance` must be finite.");
        ensure!(
            min_distance >= 10.,
            "`min_distance` must be larger or equal to 10.0."
        );

        let max_distance = conf.and_then(|c| c.max_distance).unwrap_or(80.);
        ensure!(max_distance.is_finite(), "`max_distance` must be finite.");
        ensure!(
            max_distance <= 300.,
            "`max_distance` must be smaller or equal to 300.0."
        );
        ensure!(
            min_distance <= max_distance,
            "`min_distance` must be smaller or equal to `max_distance`."
        );

        let wheel_zoom_sensitivity = conf.and_then(|c| c.wheel_zoom_sensitivity).unwrap_or(1.1);
        ensure!(
            wheel_zoom_sensitivity.is_finite(),
            "`wheel_zoom_sensitivity` must be finite."
        );
        ensure!(
            wheel_zoom_sensitivity > 1.,
            "`wheel_zoom_sensitivity` must be greater than 1.0."
        );

        let touchpad_zoom_sensitivity = conf
            .and_then(|c| c.touchpad_zoom_sensitivity)
            .unwrap_or(1.01);
        ensure!(
            touchpad_zoom_sensitivity.is_finite(),
            "`touchpad_zoom_sensitivity` must be finite."
        );
        ensure!(
            touchpad_zoom_sensitivity > 1.,
            "`touchpad_zoom_sensitivity` must be greater than 1.0."
        );

        let rotation_sensitivity = conf.and_then(|c| c.rotation_sensitivity).unwrap_or(0.008);
        ensure!(
            rotation_sensitivity.is_finite(),
            "`rotation_sensitivity` must be finite."
        );
        ensure!(
            rotation_sensitivity > 0.,
            "`rotation_sensitivity` must be greater than 0.0."
        );

        Ok(Self {
            move_margin: LogicalPixel::new(move_margin),
            min_distance: Metre::new(min_distance),
            max_distance: Metre::new(max_distance),
            wheel_zoom_sensitivity,
            touchpad_zoom_sensitivity,
            rotation_sensitivity: InverseLogicalPixel::new(rotation_sensitivity),
        })
    }
}
