//! This crate implements functionality around map object handling, mostly
//! object (de)spawning, object asset caching and pre-loading.

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use cache::CachePlugin;
pub use cache::ObjectCache;
pub use collider::{ColliderCache, ObjectCollider};
pub use ichnography::{Ichnography, IchnographyCache};
pub use spawner::SpawnBundle;
use spawner::SpawnerPlugin;

mod cache;
mod collider;
mod ichnography;
mod loader;
mod spawner;

pub struct ObjectsPluginGroup;

impl PluginGroup for ObjectsPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(CachePlugin).add(SpawnerPlugin);
    }
}
