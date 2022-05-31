#![allow(rustdoc::private_intra_doc_links)]
//! This library implements a Bevy plugin for optimal path finding on the game
//! map.
//!
//! The plugin [`PathingPlugin`] registers systems which
//! automatically update the path finder when static solid objects are added or
//! removed from the world.
//!
//! When [`UpdateEntityPath`] event is sent, the entity paths is automatically
//! (re)planned.
//!
//! # World Update
//!
//! * Each solid static object's ichnography (a convex polygon) is offset by
//!   some amount. See [`crate::exclusion`].
//!
//! * Overlapping polygons from the previous steps are merged -- their convex
//!   hull is used. These are called exclusion areas.
//!
//! * Whole map (surface) is triangulated with Constrained Delaunay
//!   triangulation (CDT). All edges from the exclusion areas are used as
//!   constrains. See [`crate::triangulation`].
//!
//! * Triangles from inside the exclusion areas are dropped, remaining
//!   triangles are used in successive steps.
//!
//! * A visibility sub-graph is created. The each triangle edge is connected
//!   with all neighboring triangle edges. See
//!   [`crate::finder::PathFinder::from_triangles`].
//!
//! # Path Search
//!
//! * Neighboring nodes (triangle edges) to the starting and target points are
//!   found. See [`crate::finder`].
//!
//! * Visibility graph is traversed with a modified Dijkstra's algorithm. See
//!   [`crate::dijkstra`]. Funnel algorithm is embedded into the A* algorithm
//!   so path funneling can be gradually applied during the graph traversal.
//!   See [`crate::funnel`].

mod chain;
mod dijkstra;
mod exclusion;
mod finder;
mod funnel;
mod geometry;
mod graph;
mod path;
mod systems;
mod triangulation;
mod utils;

pub use systems::{PathingPlugin, UpdateEntityPath};
