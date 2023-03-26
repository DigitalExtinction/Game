//! This crate implements functionality around map object handling, mostly
//! object asset caching and pre-loading.

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
pub use cannon::LaserCannon;
pub use collection::AssetCollection;
pub use collider::ObjectCollider;
pub use flight::Flight;
use health::HealthPlugin;
pub use health::{Health, InitialHealths};
pub use ichnography::{Ichnography, EXCLUSION_OFFSET};
pub use scenes::Scenes;
use scenes::ScenesPlugin;
use solids::SolidsPlugin;
pub use solids::{SolidObject, SolidObjects};

mod cannon;
mod collection;
mod collider;
mod factory;
mod flight;
mod health;
mod ichnography;
mod names;
mod scenes;
mod solids;

pub struct ObjectsPluginGroup;

impl PluginGroup for ObjectsPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(ScenesPlugin)
            .add(SolidsPlugin)
            .add(HealthPlugin)
    }
}
