#![allow(rustdoc::private_intra_doc_links)]
//! This crate implements spatial indexing and various spatial queries of game
//! entities.

mod precise;

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use precise::PreciseIndexPlugin;
pub use precise::{
    ColliderWithCache, EntityIndex, LocalCollider, PreciseIndexSet, QueryCollider,
    RayEntityIntersection, SpatialQuery,
};

/// Size (in world-space) of a single square tile where entities are kept.
const TILE_SIZE: f32 = 10.;

pub struct IndexPluginGroup;

impl PluginGroup for IndexPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(PreciseIndexPlugin)
    }
}
