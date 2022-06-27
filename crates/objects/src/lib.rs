//! This crate implements functionality around map object handling, mostly
//! object asset caching and pre-loading.

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use cache::CachePlugin;
pub use cache::ObjectCache;
pub use collider::{ColliderCache, ObjectCollider};
pub use ichnography::{Ichnography, IchnographyCache};

mod cache;
mod collider;
mod ichnography;
mod loader;

pub struct ObjectsPluginGroup;

impl PluginGroup for ObjectsPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(CachePlugin);
    }
}
