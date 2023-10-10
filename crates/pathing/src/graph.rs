//! This module contains implementation of a edge-based visibility graph used
//! in shortest path search on the game map.

use parry2d::shape::Segment;
use tinyvec::ArrayVec;

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
    nodes: Vec<GraphNode>,
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

    /// Prepares a graph node without any neighbours, pushes it to the graph
    /// and returns corresponding edge (node) ID.
    ///
    /// Only single node must be created when multiple triangles share an edge
    /// (have coincidental edge line segment).
    ///
    /// # Arguments
    ///
    /// * `segment` - line segment of the triangle edge.
    pub(crate) fn new_node(&mut self, segment: Segment) -> u32 {
        let id = self.nodes.len().try_into().unwrap();
        self.nodes.push(GraphNode::new(segment));
        id
    }

    /// Adds 2 neighbours accessible via one of the adjacent triangles to a
    /// graph node (triangle edge).
    ///
    /// # Arguments
    ///
    /// * `edge_id` - ID of the edge whose neighbors are added.
    ///
    /// * `triangle_id` - ID of the traversed triangle (i.e. the triangle
    ///   containing the source and target edges).
    ///
    /// * `neighbour_a_id` - edge ID of the a neighbor.
    ///
    /// * `neighbour_b_id` - edge ID of the a neighbor.
    ///
    /// # Panics
    ///
    /// Panics if `edge_id` already stores more than two neighbours.
    pub(crate) fn add_neighbours(
        &mut self,
        edge_id: u32,
        triangle_id: u32,
        neighbour_a_id: u32,
        neighbour_b_id: u32,
    ) {
        let index: usize = edge_id.try_into().unwrap();
        let node = self.nodes.get_mut(index).unwrap();
        node.add_neighbour(Step::new(neighbour_a_id, triangle_id));
        node.add_neighbour(Step::new(neighbour_b_id, triangle_id));
    }

    /// Returns a geometry of a graph node (triangle edge).
    pub(crate) fn segment(&self, edge_id: u32) -> Segment {
        let index: usize = edge_id.try_into().unwrap();
        self.nodes[index].segment()
    }

    /// Returns all neighbors of a graph node (triangle edge).
    pub(crate) fn neighbours(&self, edge_id: u32) -> &[Step] {
        let index: usize = edge_id.try_into().unwrap();
        self.nodes[index].neighbours()
    }
}

/// A node in the visibility graph.
struct GraphNode {
    segment: Segment,
    /// Graph steps to reach direct neighbors.
    neighbours: ArrayVec<[Step; 4]>,
}

impl GraphNode {
    fn new(segment: Segment) -> Self {
        Self {
            segment,
            neighbours: ArrayVec::new(),
        }
    }

    fn segment(&self) -> Segment {
        self.segment
    }

    fn neighbours(&self) -> &[Step] {
        self.neighbours.as_slice()
    }

    /// Adds a neighbor to the node.
    ///
    /// Each node can store up to 4 neighbors.
    ///
    /// # Panics
    ///
    /// * If the number of already stored neighbors is 4.
    ///
    /// * If the number of already stored triangles is 2.
    fn add_neighbour(&mut self, step: Step) {
        self.neighbours.push(step);
    }
}

/// A step in the triangle edge neighbor graph. Id est triangle traversal from
/// a set of points in the triangle (one point or a line segment) to (part of)
/// an edge of the triangle.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub(crate) struct Step {
    edge_id: u32,
    triangle_id: u32,
}

impl Step {
    pub(crate) fn new(edge_id: u32, triangle_id: u32) -> Self {
        Self {
            edge_id,
            triangle_id,
        }
    }

    /// A target edge ID (reached from neighboring edge).
    pub(crate) fn edge_id(&self) -> u32 {
        self.edge_id
    }

    /// ID of the traversed triangle (to reach [`Self::edge_id()`].
    pub(crate) fn triangle_id(&self) -> u32 {
        self.triangle_id
    }
}

#[cfg(test)]
mod tests {
    use parry2d::math::Point;

    use super::*;

    #[test]
    fn test_graph() {
        let mut graph = VisibilityGraph::new();

        let edge_id_a = graph.new_node(Segment::new(Point::new(1., 2.), Point::new(2., 3.)));
        let edge_id_b = graph.new_node(Segment::new(Point::new(2., 3.), Point::new(5., 6.)));
        let edge_id_c = graph.new_node(Segment::new(Point::new(5., 6.), Point::new(1., 2.)));
        graph.add_neighbours(edge_id_a, 1, edge_id_b, edge_id_c);
        graph.add_neighbours(edge_id_b, 1, edge_id_c, edge_id_a);

        assert_eq!(
            graph.neighbours(edge_id_a),
            &[Step::new(edge_id_b, 1), Step::new(edge_id_c, 1)]
        );
        assert_eq!(
            graph.neighbours(edge_id_b),
            &[Step::new(edge_id_c, 1), Step::new(edge_id_a, 1)]
        );
        assert_eq!(graph.neighbours(edge_id_c), &[] as &[Step]);
    }
}
