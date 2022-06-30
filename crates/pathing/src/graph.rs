//! This module contains implementation of a edge-based visibility graph used
//! in shortest path search on the game map.

use parry2d::{math::Point, shape::Segment};
use tinyvec::TinyVec;

/// Edge based visibility sub-graph.
///
/// Be careful to distinguish between triangle edges and graph edges:
///
/// * triangle edge - a geometric edge of a triangle (each triangle has 3).
///
/// * graph node - a node in a graph. In the case of `VisibilityGraph`, each
///   graph node represents a triangle edge.
///
/// * graph edge - an edge between two nodes in a graph. In the case of
///   `VisibilityGraph`, each graph edge represents direct visibility between
///   two triangle edges.
///
///   If triangle edges A and B are connected by a graph edge (i.e. are
///   neighbours in `VisibilityGraph`), then any point on triangle edge A is
///   accessible (visible) on straight line from any point on triangle B.
///
///   The opposite statement doesn't always hold: two triangle edges might be
///   fully visible one from another and not share a graph edge. However, such
///   triangles are always connected by a graph path.
pub struct VisibilityGraph {
    nodes: Vec<Node>,
}

impl VisibilityGraph {
    /// Returns a new empty visibility graph.
    pub(crate) fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Returns number of nodes in the visibility graph.
    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Creates a graph node without any neighbours, stores to the graph and
    /// returns its ID.
    ///
    /// Only single node must be created when multiple triangles share an edge
    /// (have coincidental edge line segment).
    ///
    /// # Arguments
    ///
    /// * `segment` - line segment of the triangle edge.
    pub(crate) fn new_node(&mut self, segment: Segment) -> u32 {
        let id = self.nodes.len().try_into().unwrap();
        self.nodes.push(Node::new(EdgeGeometry::new(segment)));
        id
    }

    /// Add 2 neighbours to a graph node (triangle edge).
    ///
    /// # Panics
    ///
    /// Panics if `edge_id` already stores more than two neighbours.
    pub(crate) fn add_neighbours(
        &mut self,
        edge_id: u32,
        neighbour_a_id: u32,
        neighbour_b_id: u32,
    ) {
        let index: usize = edge_id.try_into().unwrap();
        let node = self.nodes.get_mut(index).unwrap();
        node.add_neighbour(neighbour_a_id);
        node.add_neighbour(neighbour_b_id);
    }

    /// Returns a geometry of a graph node (triangle edge).
    pub(crate) fn geometry(&self, edge_id: u32) -> &EdgeGeometry {
        let index: usize = edge_id.try_into().unwrap();
        self.nodes[index].geometry()
    }

    /// Returns all neighbors of a graph node (triangle edge).
    pub(crate) fn neighbours(&self, edge_id: u32) -> &[u32] {
        let index: usize = edge_id.try_into().unwrap();
        self.nodes[index].neighbours()
    }
}

/// A node in the visibility graph.
struct Node {
    geometry: EdgeGeometry,
    /// Neighbor IDs.
    neighbours: TinyVec<[u32; 4]>,
}

impl Node {
    fn new(geometry: EdgeGeometry) -> Self {
        Self {
            geometry,
            neighbours: TinyVec::new(),
        }
    }

    fn geometry(&self) -> &EdgeGeometry {
        &self.geometry
    }

    fn neighbours(&self) -> &[u32] {
        self.neighbours.as_slice()
    }

    /// Adds a neighbor to the node.
    ///
    /// Each node can store up to 4 neighbors.
    ///
    /// # Panics
    ///
    /// Panics if the number of already stored neighbors is 4.
    fn add_neighbour(&mut self, edge_id: u32) {
        self.neighbours.push(edge_id);
    }
}

pub(crate) struct EdgeGeometry {
    segment: Segment,
    /// Middle of `segment` cached for efficiency reasons.
    midpoint: Point<f32>,
}

impl EdgeGeometry {
    fn new(segment: Segment) -> Self {
        Self {
            segment,
            midpoint: segment.a.coords.lerp(&segment.b.coords, 0.5).into(),
        }
    }

    pub(crate) fn segment(&self) -> Segment {
        self.segment
    }

    pub(crate) fn midpoint(&self) -> Point<f32> {
        self.midpoint
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph() {
        let mut graph = VisibilityGraph::new();

        let edge_id_a = graph.new_node(Segment::new(Point::new(1., 2.), Point::new(2., 3.)));
        let edge_id_b = graph.new_node(Segment::new(Point::new(2., 3.), Point::new(5., 6.)));
        let edge_id_c = graph.new_node(Segment::new(Point::new(5., 6.), Point::new(1., 2.)));
        graph.add_neighbours(edge_id_a, edge_id_b, edge_id_c);
        graph.add_neighbours(edge_id_b, edge_id_c, edge_id_a);

        assert_eq!(graph.neighbours(edge_id_a), &[edge_id_b, edge_id_c]);
        assert_eq!(graph.neighbours(edge_id_b), &[edge_id_c, edge_id_a]);
        assert_eq!(graph.neighbours(edge_id_c), &[] as &[u32]);
        assert_eq!(graph.geometry(edge_id_a).midpoint(), Point::new(1.5, 2.5));
    }
}
