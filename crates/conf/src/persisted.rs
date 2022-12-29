//! This module contains configuration object which can be (de)serialized from
//! a configuration file. It does not contain final configuration object which
//! must be build and validated from the objects here.

use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Default)]
pub(super) struct Configuration {
    pub(super) multiplayer: Option<Multiplayer>,
    pub(super) camera: Option<Camera>,
}

#[derive(Deserialize, Default)]
pub(super) struct Multiplayer {
    pub(super) server: Option<Url>,
}

#[derive(Deserialize, Default)]
pub struct Camera {
    pub(super) move_margin: Option<f32>,
    pub(super) min_distance: Option<f32>,
    pub(super) max_distance: Option<f32>,
    pub(super) wheel_zoom_sensitivity: Option<f32>,
    pub(super) touchpad_zoom_sensitivity: Option<f32>,
    pub(super) rotation_sensitivity: Option<f32>,
}
