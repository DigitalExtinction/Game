//! This crate implements functionality around map object handling, mostly
//! object asset caching and pre-loading.

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use cache::CachePlugin;
pub use cache::ObjectCache;
pub use cannon::LaserCannon;
pub use collider::{ColliderCache, ObjectCollider};
pub use flight::Flight;
use health::HealthPlugin;
pub use health::{Health, InitialHealths};
pub use ichnography::{Ichnography, IchnographyCache, EXCLUSION_OFFSET};

mod cache;
mod cannon;
mod collider;
mod factory;
mod flight;
mod health;
mod ichnography;
mod loader;

pub struct ObjectsPluginGroup;

impl PluginGroup for ObjectsPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CachePlugin)
            .add(HealthPlugin)
    }
}
