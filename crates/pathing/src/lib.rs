#![allow(rustdoc::private_intra_doc_links)]
//! This library implements a Bevy plugin for optimal path finding on the game
//! map.
//!
//! When [`UpdateEntityPath`] event is sent, the entity paths is automatically
//! (re)planned.
//!
//!
//! # Path Search
//!
//! * Neighboring nodes (triangle edges) to the starting and target points are
//!   found. See [`crate::finder`].
//!
//! * Visibility graph is traversed with a modified Dijkstra's algorithm. See
//!   [`crate::dijkstra`]. Funnel algorithm is embedded into the algorithm so
//!   path funneling can be gradually applied during the graph traversal. See
//!   [`crate::funnel`].

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
pub use fplugin::create_finder;
use fplugin::FinderPlugin;
pub use path::ScheduledPath;
use pplugin::PathingPlugin;
pub use pplugin::UpdateEntityPath;
pub use query::{PathQueryProps, PathTarget};

pub struct PathingPluginGroup;

impl PluginGroup for PathingPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(FinderPlugin).add(PathingPlugin);
    }
}
