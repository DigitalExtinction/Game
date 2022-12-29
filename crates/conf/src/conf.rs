//! This module implements final (i.e. parsed and validated) game configuration
//! objects and their building from persistent configuration objects.

use anyhow::{bail, Context, Error, Result};
use bevy::prelude::Resource;
use url::Url;

use crate::persisted::*;

/// Top-level game configuration object.
#[derive(Resource)]
pub struct Configuration {
    multiplayer: MultiplayerConf,
}

impl Configuration {
    pub fn multiplayer(&self) -> &MultiplayerConf {
        &self.multiplayer
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
