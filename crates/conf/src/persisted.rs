//! This module contains configuration object which can be (de)serialized from
//! a configuration file. It does not contain final configuration object which
//! must be build and validated from the objects here.

use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Default)]
pub(super) struct Configuration {
    pub(super) multiplayer: Option<Multiplayer>,
}

#[derive(Deserialize, Default)]
pub(super) struct Multiplayer {
    pub(super) server: Option<Url>,
}
