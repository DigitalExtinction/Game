#![allow(rustdoc::private_intra_doc_links)]
//! This library implements a Bevy plugin for any angle path finding on the
//! game map.

mod chain;
mod dijkstra;
mod exclusion;
mod finder;
mod fplugin;
mod funnel;
mod geometry;
mod graph;
mod path;
mod pplugin;
mod query;
mod triangulation;
mod utils;

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
pub use exclusion::ExclusionArea;
pub use fplugin::create_finder;
use fplugin::FinderPlugin;
pub use path::ScheduledPath;
use pplugin::PathingPlugin;
pub use pplugin::UpdateEntityPathEvent;
pub use query::{PathQueryProps, PathTarget};

pub struct PathingPluginGroup;

impl PluginGroup for PathingPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(FinderPlugin)
            .add(PathingPlugin)
    }
}
