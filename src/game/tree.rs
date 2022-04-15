use depacked::{Item, PackedData};
use glam::Vec2;
use std::mem::MaybeUninit;

const MAX_LEAFS: usize = 32;

pub struct Tree<T> {
    bounds: Rectangle,
    root_node_id: Item<Node<T>>,
    packed_nodes: PackedData<Node<T>>,
    packed_elements: PackedData<Element<T>>,
}

impl<T> Tree<T> {
    pub fn with_capacity(capacity: usize, bounds: Rectangle) -> Self {
        let mut packed_nodes = PackedData::with_max_capacity(2 * capacity);
        let root_node_id = packed_nodes.insert(Node::Leaf(LeafNode::empty(bounds)));
        let packed_elements = PackedData::with_max_capacity(capacity);
        Self {
            bounds,
            root_node_id,
            packed_nodes,
            packed_elements,
        }
    }

    pub fn within_disc(&self, disc: Disc) -> Vec<(&T, Vec2)> {
        let mut elements = Vec::new();

        let mut stack = Vec::with_capacity(30);
        stack.push(self.root_node_id);

        while !stack.is_empty() {
            match self.packed_nodes.get(stack.pop().unwrap()) {
                Node::Inner(inode) => {
                    if inode.intersects_disc(disc) {
                        stack.push(inode.left_node_id());
                        stack.push(inode.right_node_id());
                    }
                }
                Node::Leaf(leaf) => {
                    if leaf.intersects_disc(disc) {
                        leaf.push_within_disc(disc, &self.packed_elements, &mut elements)
                    }
                }
            }
        }

        elements
    }

    pub fn insert(&mut self, data: T, position: Vec2) -> TreeItem<T> {
        self.check_point(position);
        let element_id = self.packed_elements.insert(Element::new(data, position));
        self.insert_inner(element_id, position, self.root_node_id);
        TreeItem::new(element_id)
    }

    pub fn remove(&mut self, tree_item: TreeItem<T>) {
        let element = self.packed_elements.remove(tree_item.element_id());

        let mut node_id = self.root_node_id;
        let mut merge_candidates = Vec::new();

        loop {
            match self.packed_nodes.get(node_id) {
                Node::Inner(inode) => {
                    merge_candidates.push(node_id);

                    if self
                        .packed_nodes
                        .get(inode.left_node_id())
                        .contains_point(element.position())
                    {
                        node_id = inode.left_node_id();
                    } else {
                        node_id = inode.right_node_id();
                    }
                }
                Node::Leaf(_) => {
                    let leaf = match self.packed_nodes.get_mut(node_id) {
                        Node::Leaf(leaf) => leaf,
                        Node::Inner(_) => panic!("TODO"),
                    };
                    leaf.remove(tree_item.element_id());
                    break;
                }
            }
        }

        for &merge_candidate in merge_candidates.iter().rev() {
            self.merge_if_possible(merge_candidate);
        }
    }

    pub fn update_position(&mut self, tree_item: TreeItem<T>, new_position: Vec2) {
        self.check_point(new_position);

        let element = self.packed_elements.get_mut(tree_item.element_id());
        let old_position = element.update_position(new_position);

        let mut node_id = self.root_node_id;
        let mut merge_candidates = Vec::new();
        let mut last_common_ancestor = None;

        loop {
            match self.packed_nodes.get(node_id) {
                Node::Inner(inode) => {
                    let left_node = self.packed_nodes.get(inode.left_node_id());

                    let is_old_left = left_node.contains_point(old_position);
                    if last_common_ancestor.is_none() {
                        if is_old_left != left_node.contains_point(new_position) {
                            last_common_ancestor = Some(node_id);
                        }
                    } else {
                        merge_candidates.push(node_id);
                    }

                    if is_old_left {
                        node_id = inode.left_node_id();
                    } else {
                        node_id = inode.right_node_id();
                    }
                }
                Node::Leaf(_) => {
                    if last_common_ancestor.is_some() {
                        let leaf = match self.packed_nodes.get_mut(node_id) {
                            Node::Leaf(leaf) => leaf,
                            Node::Inner(_) => panic!("TODO"),
                        };
                        leaf.remove(tree_item.element_id());
                    }
                    break;
                }
            }
        }

        if let Some(target_node_id) = last_common_ancestor {
            self.insert_inner(tree_item.element_id(), new_position, target_node_id);
        }

        for &merge_candidate in merge_candidates.iter().rev() {
            self.merge_if_possible(merge_candidate);
        }
    }

    fn check_point(&self, point: Vec2) {
        if !self.bounds.contains_point(point) {
            panic!(
                "Point {:?} is out of the tree bounds {:?}.",
                point, self.bounds
            );
        }
    }

    fn insert_inner(
        &mut self,
        element_id: Item<Element<T>>,
        position: Vec2,
        mut node_id: Item<Node<T>>,
    ) {
        loop {
            match self.packed_nodes.get(node_id) {
                Node::Inner(inode) => {
                    if self
                        .packed_nodes
                        .get(inode.left_node_id())
                        .contains_point(position)
                    {
                        node_id = inode.left_node_id();
                    } else {
                        node_id = inode.right_node_id();
                    }
                }
                Node::Leaf(_) => {
                    let leaf = match self.packed_nodes.get_mut(node_id) {
                        Node::Leaf(leaf) => leaf,
                        Node::Inner(_) => panic!("TODO"),
                    };

                    if leaf.is_full() {
                        self.split(node_id);
                    } else {
                        leaf.insert(element_id);
                        break;
                    }
                }
            }
        }
    }

    fn split(&mut self, node_id: Item<Node<T>>) {
        let rectangle = match self.packed_nodes.get(node_id) {
            Node::Leaf(leaf) => leaf.rectangle(),
            Node::Inner(_) => panic!("Cannot split a non-leaf node."),
        };
        let [left_rect, right_rect] = rectangle.get_children();

        let new_inode = Inode::with_empty_leafs(&mut self.packed_nodes, rectangle);
        let left_node_id = new_inode.left_node_id();
        let right_node_id = new_inode.right_node_id();

        let old_node = {
            let mut node = Node::Inner(new_inode);
            std::mem::swap(&mut node, self.packed_nodes.get_mut(node_id));
            node
        };

        match old_node {
            Node::Leaf(mut leaf) => leaf.move_to_split(
                left_rect,
                left_node_id,
                right_rect,
                right_node_id,
                &self.packed_elements,
                &mut self.packed_nodes,
            ),
            Node::Inner(_) => panic!("Cannot split a non-leaf node."),
        }
    }

    fn merge_if_possible(&mut self, parent_node_id: Item<Node<T>>) -> bool {
        let parent_inode = match self.packed_nodes.get(parent_node_id) {
            Node::Leaf(_) => return false,
            Node::Inner(inode) => inode,
        };
        let left_node_id = parent_inode.left_node_id();
        let right_node_id = parent_inode.right_node_id();

        let (mut left_node, right_node) = match self.packed_nodes.get(left_node_id) {
            Node::Leaf(left_leaf_node) => match self.packed_nodes.get(right_node_id) {
                Node::Leaf(right_leaf_node) => {
                    if left_leaf_node.size() + right_leaf_node.size() > MAX_LEAFS {
                        return false;
                    }
                    (
                        self.packed_nodes.remove(left_node_id),
                        self.packed_nodes.remove(right_node_id),
                    )
                }
                Node::Inner(_) => return false,
            },
            Node::Inner(_) => return false,
        };

        if let Node::Leaf(mut right_leaf_node) = right_node {
            if let Node::Leaf(left_leaf_node) = &mut left_node {
                right_leaf_node.move_to(left_leaf_node);
            }
        }

        let parent_node = self.packed_nodes.get_mut(parent_node_id);
        std::mem::swap(parent_node, &mut left_node);

        true
    }
}

pub struct TreeItem<T> {
    element_id: Item<Element<T>>,
}

impl<T> TreeItem<T> {
    fn new(element_id: Item<Element<T>>) -> Self {
        Self { element_id }
    }

    #[inline]
    fn element_id(&self) -> Item<Element<T>> {
        self.element_id
    }
}

enum Node<T> {
    Inner(Inode<T>),
    Leaf(LeafNode<T>),
}

impl<T> Node<T> {
    #[inline]
    fn contains_point(&self, point: Vec2) -> bool {
        match self {
            Self::Inner(inode) => inode.contains_point(point),
            Self::Leaf(leaf) => leaf.contains_point(point),
        }
    }
}

struct Inode<T> {
    child_ids: [Item<Node<T>>; 2],
    rectangle: Rectangle,
}

impl<T> Inode<T> {
    fn with_empty_leafs(packed_nodes: &mut PackedData<Node<T>>, rectangle: Rectangle) -> Self {
        let [left_rect, right_rect] = rectangle.get_children();
        let left_node_id = packed_nodes.insert(Node::Leaf(LeafNode::empty(left_rect)));
        let right_nod_id = packed_nodes.insert(Node::Leaf(LeafNode::empty(right_rect)));
        Self {
            child_ids: [left_node_id, right_nod_id],
            rectangle,
        }
    }

    #[inline]
    fn left_node_id(&self) -> Item<Node<T>> {
        self.child_ids[0]
    }

    #[inline]
    fn right_node_id(&self) -> Item<Node<T>> {
        self.child_ids[1]
    }

    #[inline]
    fn intersects_disc(&self, disc: Disc) -> bool {
        self.rectangle.intersects_disc(disc)
    }

    #[inline]
    fn contains_point(&self, point: Vec2) -> bool {
        self.rectangle.contains_point(point)
    }
}

struct LeafNode<T> {
    size: usize,
    element_ids: [Item<Element<T>>; MAX_LEAFS],
    rectangle: Rectangle,
}

impl<T> LeafNode<T> {
    fn empty(rectangle: Rectangle) -> Self {
        // This is sound because size is set to 0 and elements beyond size are
        // never accessed
        unsafe {
            Self {
                size: 0,
                element_ids: MaybeUninit::uninit().assume_init(),
                rectangle,
            }
        }
    }

    #[inline]
    fn size(&self) -> usize {
        self.size
    }

    #[inline]
    fn is_full(&self) -> bool {
        self.size >= MAX_LEAFS
    }

    #[inline]
    fn rectangle(&self) -> Rectangle {
        self.rectangle
    }

    #[inline]
    fn intersects_disc(&self, disc: Disc) -> bool {
        self.rectangle.intersects_disc(disc)
    }

    #[inline]
    fn contains_point(&self, point: Vec2) -> bool {
        self.rectangle.contains_point(point)
    }

    fn push_within_disc<'a>(
        &self,
        disc: Disc,
        packed_elements: &'a PackedData<Element<T>>,
        target: &mut Vec<(&'a T, Vec2)>,
    ) {
        for i in 0..self.size {
            let element = packed_elements.get(self.element_ids[i]);
            if element.is_within_disc(disc) {
                target.push((element.data(), element.position()));
            }
        }
    }

    fn insert(&mut self, element_id: Item<Element<T>>) {
        if self.size == self.element_ids.len() {
            panic!("Cannot insert the child, the node is full.");
        }
        self.element_ids[self.size] = element_id;
        self.size += 1;
    }

    fn remove(&mut self, element_id: Item<Element<T>>) {
        if self.size == 0 {
            panic!("Cannot remove the child, the node is empty.");
        }

        let mut index = 0;
        while self.element_ids[index] != element_id {
            index += 1;
            if index >= self.size {
                panic!("Cannot remove the element, it is not a child of this node.");
            }
        }
        self.element_ids.swap(index, self.size - 1);
        self.size -= 1;
    }

    fn move_to(&mut self, other: &mut LeafNode<T>) {
        for i in 0..self.size {
            other.insert(self.element_ids[i]);
        }
        self.size = 0;
    }

    fn move_to_split(
        &mut self,
        left_rectangle: Rectangle,
        left_node_id: Item<Node<T>>,
        right_rectangle: Rectangle,
        right_node_id: Item<Node<T>>,
        packed_elements: &PackedData<Element<T>>,
        packed_nodes: &mut PackedData<Node<T>>,
    ) {
        for i in 0..self.size {
            let element_id = self.element_ids[i];
            let element = packed_elements.get(element_id);
            let target_node_id = if left_rectangle.contains_point(element.position()) {
                left_node_id
            } else {
                debug_assert!(right_rectangle.contains_point(element.position()));
                right_node_id
            };
            match packed_nodes.get_mut(target_node_id) {
                Node::Leaf(leaf) => {
                    leaf.insert(element_id);
                }
                Node::Inner(_) => panic!("Cannot move elements to non-leaf target node."),
            }
        }
    }
}

struct Element<T> {
    data: T,
    position: Vec2,
}

impl<T> Element<T> {
    fn new(data: T, position: Vec2) -> Self {
        Self { data, position }
    }

    #[inline]
    fn position(&self) -> Vec2 {
        self.position
    }

    #[inline]
    fn data(&self) -> &T {
        &self.data
    }

    fn update_position(&mut self, new_position: Vec2) -> Vec2 {
        let old_position = self.position;
        self.position = new_position;
        old_position
    }

    #[inline]
    fn is_within_disc(&self, disc: Disc) -> bool {
        // distance_squared is faster as it avoid computation of square root
        self.position.distance_squared(disc.center()) <= disc.radius_squared()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rectangle {
    min: Vec2,
    max: Vec2,
}

impl Rectangle {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        let dimensions = max - min;
        if dimensions.min_element() <= 0. {
            panic!("Cannot create rectangle of non-positive size.");
        }

        Self { min, max }
    }

    #[inline]
    fn long_axis(&self) -> Axis {
        let dimensions = self.max - self.min;
        if dimensions.x >= dimensions.y {
            Axis::X
        } else {
            Axis::Y
        }
    }

    #[inline]
    fn intersects_disc(&self, disc: Disc) -> bool {
        // distance_squared is faster as it avoid computation of square root
        disc.center()
            .clamp(self.min, self.max)
            .distance_squared(disc.center())
            <= disc.radius_squared()
    }

    #[inline]
    fn contains_point(&self, point: Vec2) -> bool {
        // It contains points on all edges to cover points on edges of tree
        // (root node). Rectangles to the top and left have priority.
        self.min.cmple(point).all() && self.max.cmpge(point).all()
    }

    fn get_children(&self) -> [Self; 2] {
        match self.long_axis() {
            Axis::X => {
                let middle = (self.min.x + self.max.x) / 2.;
                [
                    Self {
                        min: self.min,
                        max: Vec2::new(middle, self.max.y),
                    },
                    Self {
                        min: Vec2::new(middle, self.min.y),
                        max: self.max,
                    },
                ]
            }
            Axis::Y => {
                let middle = (self.min.y + self.max.y) / 2.;
                [
                    Self {
                        min: self.min,
                        max: Vec2::new(self.max.x, middle),
                    },
                    Self {
                        min: Vec2::new(self.min.x, middle),
                        max: self.max,
                    },
                ]
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Axis {
    X,
    Y,
}

#[derive(Debug, Clone, Copy)]
pub struct Disc {
    center: Vec2,
    radius_squared: f32,
}

impl Disc {
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self {
            center,
            radius_squared: radius.powi(2),
        }
    }

    #[inline]
    fn center(&self) -> Vec2 {
        self.center
    }

    #[inline]
    fn radius_squared(&self) -> f32 {
        self.radius_squared
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree() {
        let mut tree = Tree::with_capacity(
            1000,
            Rectangle::new(Vec2::new(-100., -50.), Vec2::new(200., 100.)),
        );

        // Add a few points very close together.
        let almost_in_center_item = tree.insert(1, Vec2::new(0.0001, 0.0001));
        tree.insert(2, Vec2::new(0.0002, 0.0002));
        tree.insert(3, Vec2::new(0.0003, 0.0003));

        // Test corners of the tree
        tree.insert(4, Vec2::new(-100., -50.));
        tree.insert(5, Vec2::new(-100., 100.));
        tree.insert(6, Vec2::new(200., -50.));
        tree.insert(7, Vec2::new(200., 100.));

        // Add many more points.
        let mut tree_items = Vec::new();
        let mut index = 10;
        for x in -10..=10 {
            for y in -5..=5 {
                let tree_item = tree.insert(index, Vec2::new((x * 10) as f32, (y * 10) as f32));
                tree_items.push(tree_item);
                index += 1;
            }
        }

        // assert_eq!(tree.packed_nodes.len(), 151);
        // assert_eq!(tree.packed_elements.len(), 238);

        let neighbours = tree.within_disc(Disc::new(Vec2::new(0.5, 0.5), 0.01));
        assert_eq!(neighbours.len(), 0);

        let neighbours = tree.within_disc(Disc::new(Vec2::new(0.5, 0.5), 1.2));
        assert_eq!(neighbours.len(), 4);
        assert_eq!(*neighbours[0].0, 1);
        assert_eq!(neighbours[0].1, Vec2::new(0.0001, 0.0001));
        assert_eq!(*neighbours[1].0, 2);
        assert_eq!(*neighbours[2].0, 3);
        assert_eq!(*neighbours[3].0, 125);
        assert_eq!(neighbours[3].1, Vec2::ZERO);

        // Test points in the corner of the tree.
        let neighbours = tree.within_disc(Disc::new(Vec2::new(200., 100.), 1.2));
        assert_eq!(neighbours.len(), 1);
        assert_eq!(*neighbours[0].0, 7);
        assert_eq!(neighbours[0].1, Vec2::new(200., 100.));

        // Test point moving
        let neighbours = tree.within_disc(Disc::new(Vec2::ZERO, 1.));
        assert_eq!(neighbours.len(), 4);
        let neighbours = tree.within_disc(Disc::new(Vec2::new(21.21, 22.21), 0.1));
        assert_eq!(neighbours.len(), 0);
        tree.update_position(almost_in_center_item, Vec2::new(21.2, 22.2));
        let neighbours = tree.within_disc(Disc::new(Vec2::ZERO, 1.));
        assert_eq!(neighbours.len(), 3);
        let neighbours = tree.within_disc(Disc::new(Vec2::new(21.21, 22.21), 0.1));
        assert_eq!(neighbours.len(), 1);

        // Test point removal.
        for tree_item in tree_items {
            tree.remove(tree_item);
        }
        assert_eq!(tree.packed_nodes.len(), 5);
        assert_eq!(tree.packed_elements.len(), 7);
        let neighbours = tree.within_disc(Disc::new(Vec2::ZERO, 1.));
        assert_eq!(neighbours.len(), 2);
    }

    // #[test]
    // fn test_leaf_node_move_to() {
    //     let mut packed_nodes = PackedData::with_max_capacity(4);
    //     let mut packed_elements = PackedData::with_max_capacity(4);

    //     let element_id_a = packed_elements.insert(Element::new(1, Vec2::new(1., 1.)));
    //     let element_id_b = packed_elements.insert(Element::new(2, Vec2::new(7., 8.)));
    //     let element_id_c = packed_elements.insert(Element::new(3, Vec2::new(17., 8.)));

    //     let mut leaf_node = LeafNode::empty();
    //     leaf_node.insert(element_id_a);
    //     leaf_node.insert(element_id_b);
    //     leaf_node.insert(element_id_c);

    //     let leaf_node_id_a = packed_nodes.insert(Node::Leaf(LeafNode::empty()));
    //     let rectangle_node_a = RectangleNode::new(
    //         Rectangle::new(Vec2::new(0., 0.), Vec2::new(10., 10.)),
    //         leaf_node_id_a,
    //     );
    //     let leaf_node_id_b = packed_nodes.insert(Node::Leaf(LeafNode::empty()));
    //     let rectangle_node_b = RectangleNode::new(
    //         Rectangle::new(Vec2::new(10., 0.), Vec2::new(20., 10.)),
    //         leaf_node_id_b,
    //     );

    //     leaf_node.move_to_split(
    //         rectangle_node_a,
    //         rectangle_node_b,
    //         &packed_elements,
    //         &mut packed_nodes,
    //     );

    //     let left = match packed_nodes.get(leaf_node_id_a) {
    //         Node::Leaf(leaf) => leaf,
    //         Node::Inner(_) => panic!("Expected leaf node."),
    //     };
    //     assert_eq!(left.size(), 2);
    //     assert_eq!(left.element_ids[0], element_id_a);
    //     assert_eq!(left.element_ids[1], element_id_b);

    //     let right = match packed_nodes.get(leaf_node_id_b) {
    //         Node::Leaf(leaf) => leaf,
    //         Node::Inner(_) => panic!("Expected leaf node."),
    //     };
    //     assert_eq!(right.size(), 1);
    //     assert_eq!(right.element_ids[0], element_id_c);
    // }

    // #[test]
    // fn test_leaf_node() {
    //     let mut packed_elements = PackedData::with_max_capacity(3);
    //     let element_id_a = packed_elements.insert(Element::new(1, Vec2::new(-100., -100.)));
    //     let element_id_b = packed_elements.insert(Element::new(2, Vec2::new(7., 8.)));
    //     let element_id_c = packed_elements.insert(Element::new(3, Vec2::new(17., 8.)));

    //     let mut leaf_node = LeafNode::empty();
    //     leaf_node.insert(element_id_a);
    //     leaf_node.insert(element_id_b);
    //     leaf_node.insert(element_id_c);

    //     // Test disc not matching any elements
    //     let mut target = Vec::new();
    //     leaf_node.push_within_disc(
    //         Disc::new(Vec2::new(5., 4.), 4.),
    //         &packed_elements,
    //         &mut target,
    //     );
    //     assert_eq!(target.len(), 0);

    //     // Test disc matching only a single element
    //     let mut target = Vec::new();
    //     leaf_node.push_within_disc(
    //         Disc::new(Vec2::new(5., 4.), 5.),
    //         &packed_elements,
    //         &mut target,
    //     );
    //     assert_eq!(target.len(), 1);
    //     assert_eq!(*target[0].0, 2);
    //     assert_eq!(target[0].1, Vec2::new(7., 8.));

    //     // Test all encompassing disc
    //     let mut target = Vec::new();
    //     leaf_node.push_within_disc(
    //         Disc::new(Vec2::new(5., 4.), 1000.),
    //         &packed_elements,
    //         &mut target,
    //     );
    //     assert_eq!(target.len(), 3);
    //     assert_eq!(*target[0].0, 1);
    //     assert_eq!(target[0].1, Vec2::new(-100., -100.));
    //     assert_eq!(*target[1].0, 2);
    //     assert_eq!(target[1].1, Vec2::new(7., 8.));
    //     assert_eq!(*target[2].0, 3);
    //     assert_eq!(target[2].1, Vec2::new(17., 8.));

    //     // Test removal of an element
    //     leaf_node.remove(element_id_b);
    //     let mut target = Vec::new();
    //     leaf_node.push_within_disc(
    //         Disc::new(Vec2::new(5., 4.), 1000.),
    //         &packed_elements,
    //         &mut target,
    //     );
    //     assert_eq!(target.len(), 2);
    //     assert_eq!(*target[0].0, 1);
    //     assert_eq!(target[0].1, Vec2::new(-100., -100.));
    //     assert_eq!(*target[1].0, 3);
    //     assert_eq!(target[1].1, Vec2::new(17., 8.));

    //     // Test empty node after removal of all nodes
    //     leaf_node.remove(element_id_a);
    //     leaf_node.remove(element_id_c);
    //     let mut target = Vec::new();
    //     leaf_node.push_within_disc(
    //         Disc::new(Vec2::new(5., 4.), 1000.),
    //         &packed_elements,
    //         &mut target,
    //     );
    //     assert_eq!(target.len(), 0);
    // }

    // #[test]
    // fn test_element() {
    //     let element = Element::new(1, Vec2::new(7., 8.));
    //     assert!(element.is_within_disc(Disc::new(Vec2::new(7., 8.), 0.0001)));
    //     assert!(element.is_within_disc(Disc::new(Vec2::new(6., 9.), 1.5)));
    //     assert!(!element.is_within_disc(Disc::new(Vec2::new(6., 9.), 1.)));
    // }

    // #[test]
    // fn test_rectangle() {
    //     let rectangle = Rectangle::new(Vec2::new(1., 2.), Vec2::new(5., 3.));
    //     assert_eq!(rectangle.long_axis(), Axis::X);

    //     let [left_child, right_child] = rectangle.get_children();
    //     assert_eq!(
    //         left_child,
    //         Rectangle::new(Vec2::new(1., 2.), Vec2::new(3., 3.))
    //     );
    //     assert_eq!(
    //         right_child,
    //         Rectangle::new(Vec2::new(3., 2.), Vec2::new(5., 3.))
    //     );

    //     assert!(rectangle.intersects_disc(Disc::new(Vec2::new(1.5, 2.5), 0.001)));
    //     assert!(rectangle.intersects_disc(Disc::new(Vec2::new(-5., -9.), 13.)));
    //     assert!(!rectangle.intersects_disc(Disc::new(Vec2::new(-5., -9.), 12.)));

    //     let rectangle = Rectangle::new(Vec2::new(1., 2.), Vec2::new(5., 12.));
    //     assert_eq!(rectangle.long_axis(), Axis::Y);

    //     let [left_child, right_child] = rectangle.get_children();
    //     assert_eq!(
    //         left_child,
    //         Rectangle::new(Vec2::new(1., 2.), Vec2::new(5., 7.))
    //     );
    //     assert_eq!(
    //         right_child,
    //         Rectangle::new(Vec2::new(1., 7.), Vec2::new(5., 12.))
    //     );
    // }

    // #[test]
    // fn test_disc() {
    //     let disc = Disc::new(Vec2::new(2.5, 5.5), 6.);
    //     assert_eq!(disc.center(), Vec2::new(2.5, 5.5));
    //     assert_eq!(disc.radius_squared(), 36.);
    // }
}
