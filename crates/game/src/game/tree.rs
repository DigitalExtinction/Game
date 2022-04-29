use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem::MaybeUninit,
};

use depacked::{Item, PackedData};
use glam::Vec2;

pub const MAX_LEAFS: usize = 50;
const MAX_LEAFS_FOR_MERGE: usize = 40;
pub const MAX_DEPTH: usize = 16;

pub struct Tree<T>
where
    T: Hash,
{
    bounds: Rectangle,
    root_inode_id: Item<Inode<T>>,
    packed_inodes: PackedData<Inode<T>>,
    packed_lnodes: PackedData<LeafNode<T>>,
}

impl<T> Tree<T>
where
    T: Hash,
{
    /// Create a new 2D tree with an expected maximum capacity.
    ///
    /// Element inserting, removal and position updating performance may
    /// deteriorate if more than expected capacity nodes are inserted.
    pub fn with_capacity(capacity: usize, bounds: Rectangle) -> Self {
        let lnodes_capacity = 4 * capacity / MAX_LEAFS + 1;
        let inodes_capacity = lnodes_capacity / 4 + 1;

        let mut packed_lnodes = PackedData::with_max_capacity(lnodes_capacity);
        let mut packed_inodes = PackedData::with_max_capacity(inodes_capacity);
        let root_inode_id = packed_inodes.insert(Inode::with_empty_leafs(&mut packed_lnodes));

        Self {
            bounds,
            root_inode_id,
            packed_inodes,
            packed_lnodes,
        }
    }

    /// Return all elements which are inside or on the edge of a disc. The
    /// elements are not returned in sorted order.
    pub fn within_disc(&self, disc: Disc) -> Vec<(&T, Vec2)> {
        let mut elements = Vec::new();

        let mut stack = vec![(Node::Inner(self.root_inode_id), self.bounds)];
        while !stack.is_empty() {
            let (node, node_bounds) = stack.pop().unwrap();

            match node {
                Node::Inner(inode_id) => {
                    let inode = self.packed_inodes.get(inode_id);
                    for (i, &child_rect) in node_bounds.get_children().iter().enumerate() {
                        if child_rect.intersects_disc(disc) {
                            stack.push((inode.get_child(i), child_rect));
                        }
                    }
                }
                Node::Leaf(leaf_node_id) => self
                    .packed_lnodes
                    .get(leaf_node_id)
                    .push_within_disc(disc, &mut elements),
            }
        }

        elements
    }

    /// Insert a new element into the tree.
    ///
    /// # Panics
    ///
    /// Panics if `position` is out of tree bounds.
    ///
    /// Panics if the insertion would lead to construction of a 2D tree with
    /// depth surpassing `MAX_DEPTH`. This might happen when more than
    /// `MAX_LEAFS` elements are on the same position or very close to each
    /// other.
    pub fn insert(&mut self, data: T, position: Vec2) -> TreeItem<T> {
        self.check_point(position);

        let mut s = DefaultHasher::new();
        data.hash(&mut s);
        let hash = s.finish();

        self.insert_inner(
            Element::new(data, position, hash),
            0,
            self.root_inode_id,
            self.bounds,
        );
        TreeItem::new(hash, position)
    }

    /// Remove an element from the tree.
    ///
    /// # Panics
    ///
    /// This method panics if the element is not in the tree.
    pub fn remove(&mut self, tree_item: TreeItem<T>) {
        let mut next_node = Node::Inner(self.root_inode_id);
        let mut next_rect = self.bounds;

        let mut merge_candidates = Vec::new();

        loop {
            match next_node {
                Node::Inner(inode_id) => {
                    let inode = self.packed_inodes.get(inode_id);
                    let child_rects = next_rect.get_children();

                    for (i, &rect) in child_rects.iter().enumerate() {
                        if rect.contains_point(tree_item.position()) {
                            next_node = inode.get_child(i);
                            next_rect = rect;
                            merge_candidates.push((inode_id, i));
                            break;
                        }
                    }
                }
                Node::Leaf(leaf_node_id) => {
                    let leaf_node = self.packed_lnodes.get_mut(leaf_node_id);
                    leaf_node.remove(leaf_node.find(&tree_item));
                    break;
                }
            }
        }

        self.merge_if_possible_chain(&merge_candidates);
    }

    /// Update position of an element.
    ///
    /// # Panics
    ///
    /// Panics if `position` is out of tree bounds.
    ///
    /// Panics if the insertion would lead to construction of a 2D tree with
    /// depth surpassing `MAX_DEPTH`. This might happen when more than
    /// `MAX_LEAFS` elements are on the same position or very close to each
    /// other.
    pub fn update_position(&mut self, tree_item: &mut TreeItem<T>, new_position: Vec2) {
        self.check_point(new_position);

        let old_position = tree_item.position();
        tree_item.update_position(new_position);

        let mut next_node = Node::Inner(self.root_inode_id);
        let mut next_rect = self.bounds;
        let mut merge_candidates = Vec::new();
        let mut last_common_ancestor = None;
        let mut moved_element = None;

        for depth in 0.. {
            match next_node {
                Node::Inner(inode_id) => {
                    let (child_index, &child_rect) = next_rect
                        .get_children()
                        .iter()
                        .enumerate()
                        .find(|(_, &rect)| rect.contains_point(old_position))
                        .unwrap();

                    if last_common_ancestor.is_none() {
                        if !child_rect.contains_point(new_position) {
                            last_common_ancestor = Some((depth, inode_id, next_rect));
                        }
                    } else {
                        merge_candidates.push((inode_id, child_index));
                    }

                    next_node = self.packed_inodes.get(inode_id).get_child(child_index);
                    next_rect = child_rect;
                }
                Node::Leaf(leaf_node_id) => {
                    let leaf_node = self.packed_lnodes.get_mut(leaf_node_id);
                    let element_index = leaf_node.find(tree_item);

                    if last_common_ancestor.is_some() {
                        let mut element = leaf_node.remove(element_index);
                        element.update_position(new_position);
                        moved_element = Some(element);
                    } else {
                        leaf_node
                            .get_mut(element_index)
                            .update_position(new_position);
                    }
                    break;
                }
            }
        }

        if let Some((depth, inode_id, node_bounds)) = last_common_ancestor {
            self.insert_inner(moved_element.unwrap(), depth, inode_id, node_bounds);
        }

        self.merge_if_possible_chain(&merge_candidates);
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
        element: Element<T>,
        target_depth: usize,
        target_inode_id: Item<Inode<T>>,
        target_bounds: Rectangle,
    ) {
        let mut next_node = Node::Inner(target_inode_id);
        let mut next_rect = target_bounds;
        let mut parent = None;

        for depth in target_depth.. {
            if depth > MAX_DEPTH {
                panic!("Maximum tree depth {} reached.", MAX_DEPTH);
            }

            match next_node {
                Node::Inner(inode_id) => {
                    let (child_index, &child_rect) = next_rect
                        .get_children()
                        .iter()
                        .enumerate()
                        .find(|(_, &rect)| rect.contains_point(element.position()))
                        .unwrap();

                    parent = Some((inode_id, child_index));
                    next_node = self.packed_inodes.get(inode_id).get_child(child_index);
                    next_rect = child_rect;
                }
                Node::Leaf(leaf_node_id) => {
                    let leaf = self.packed_lnodes.get_mut(leaf_node_id);
                    if leaf.is_full() {
                        let (parent_inode_id, child_index) = parent.unwrap();
                        next_node = Node::Inner(self.split(
                            parent_inode_id,
                            child_index,
                            leaf_node_id,
                            next_rect,
                        ));
                    } else {
                        leaf.insert(element);
                        break;
                    }
                }
            }
        }
    }

    fn split(
        &mut self,
        parent_inode_id: Item<Inode<T>>,
        child_index: usize,
        leaf_node_id: Item<LeafNode<T>>,
        leaf_node_bounds: Rectangle,
    ) -> Item<Inode<T>> {
        let old_lnode = self.packed_lnodes.remove(leaf_node_id);
        let new_inode = Inode::with_empty_leafs(&mut self.packed_lnodes);
        old_lnode.move_to_split(&new_inode, leaf_node_bounds, &mut self.packed_lnodes);
        let new_inode_id = self.packed_inodes.insert(new_inode);
        self.packed_inodes
            .get_mut(parent_inode_id)
            .set_child(child_index, Node::Inner(new_inode_id));
        new_inode_id
    }

    fn merge_if_possible_chain(&mut self, merge_candidates: &[(Item<Inode<T>>, usize)]) {
        for parent_child in merge_candidates.windows(2).rev() {
            let (parent_inode_id, child_index) = parent_child[0];
            let inode_id = parent_child[1].0;
            self.merge_if_possible(parent_inode_id, child_index, inode_id);
        }
    }

    fn merge_if_possible(
        &mut self,
        parent_inode_id: Item<Inode<T>>,
        child_index: usize,
        node_id: Item<Inode<T>>,
    ) -> bool {
        let inode = self.packed_inodes.get(node_id);

        let mut leaf_node_ids = Vec::with_capacity(4);
        for child in inode.children() {
            match child {
                Node::Leaf(leaf_node_id) => leaf_node_ids.push(leaf_node_id),
                Node::Inner(_) => return false,
            }
        }

        let total_size: usize = leaf_node_ids
            .iter()
            .map(|&id| self.packed_lnodes.get(id).size())
            .sum();
        if total_size > MAX_LEAFS_FOR_MERGE {
            return false;
        }

        let mut new_leaf_node = LeafNode::empty();
        self.packed_inodes.remove(node_id);
        for leaf_node_id in leaf_node_ids {
            let leaf_node = self.packed_lnodes.remove(leaf_node_id);
            leaf_node.move_to(&mut new_leaf_node);
        }

        let leaf_node_id = self.packed_lnodes.insert(new_leaf_node);
        self.packed_inodes
            .get_mut(parent_inode_id)
            .set_child(child_index, Node::Leaf(leaf_node_id));
        true
    }
}

pub struct TreeItem<T> {
    hash: u64,
    position: Vec2,
    _marker: PhantomData<T>,
}

impl<T> TreeItem<T> {
    fn new(hash: u64, position: Vec2) -> Self {
        Self {
            hash,
            position,
            _marker: PhantomData,
        }
    }

    #[inline]
    fn position(&self) -> Vec2 {
        self.position
    }

    #[inline]
    fn hash(&self) -> u64 {
        self.hash
    }

    #[inline]
    fn update_position(&mut self, new_position: Vec2) {
        self.position = new_position;
    }
}

enum Node<T> {
    Inner(Item<Inode<T>>),
    Leaf(Item<LeafNode<T>>),
}

impl<T> Clone for Node<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Inner(i) => Self::Inner(*i),
            Self::Leaf(l) => Self::Leaf(*l),
        }
    }
}

impl<T> Copy for Node<T> {}

struct Inode<T> {
    children: [Node<T>; 4],
}

impl<T> Inode<T> {
    fn with_empty_leafs(packed_lnodes: &mut PackedData<LeafNode<T>>) -> Self {
        Self {
            children: [
                Node::Leaf(packed_lnodes.insert(LeafNode::empty())),
                Node::Leaf(packed_lnodes.insert(LeafNode::empty())),
                Node::Leaf(packed_lnodes.insert(LeafNode::empty())),
                Node::Leaf(packed_lnodes.insert(LeafNode::empty())),
            ],
        }
    }

    #[inline]
    fn children(&self) -> [Node<T>; 4] {
        self.children
    }

    #[inline]
    fn get_child(&self, index: usize) -> Node<T> {
        self.children[index]
    }

    #[inline]
    fn set_child(&mut self, index: usize, child: Node<T>) {
        self.children[index] = child;
    }
}

struct LeafNode<T> {
    size: usize,
    elements: [Element<T>; MAX_LEAFS],
}

impl<T> LeafNode<T> {
    fn empty() -> Self {
        Self {
            size: 0,
            // This is sound because size is set to 0 and elements beyond size
            // are never accessed
            elements: unsafe { MaybeUninit::zeroed().assume_init() },
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

    fn push_within_disc<'a>(&'a self, disc: Disc, target: &mut Vec<(&'a T, Vec2)>) {
        for i in 0..self.size {
            let element = unsafe { self.elements.get_unchecked(i) };
            if element.is_within_disc(disc) {
                target.push((element.data(), element.position()));
            }
        }
    }

    fn insert(&mut self, element: Element<T>) {
        if self.size == self.elements.len() {
            panic!("Cannot insert the child, the node is full.");
        }
        self.elements[self.size] = element;
        self.size += 1;
    }

    fn find(&self, item: &TreeItem<T>) -> usize {
        if self.size == 0 {
            panic!("Child not found, the node is empty.");
        }

        let mut index = 0;
        while !self.elements[index].matches_tree_item(item) {
            index += 1;
            if index >= self.size {
                panic!("Child not found.");
            }
        }
        index
    }

    fn get_mut(&mut self, index: usize) -> &mut Element<T> {
        if index >= self.size {
            panic!("Index {} is beyond leaf node size {}.", index, self.size);
        }
        &mut self.elements[index]
    }

    fn remove(&mut self, index: usize) -> Element<T> {
        if index >= self.size {
            panic!("Index {} is beyond leaf node size {}.", index, self.size);
        }

        self.elements.swap(index, self.size - 1);
        self.size -= 1;

        let mut element = unsafe { MaybeUninit::zeroed().assume_init() };
        std::mem::swap(&mut element, &mut self.elements[self.size]);
        element
    }

    fn move_to(self, other: &mut LeafNode<T>) {
        for element in self.elements.into_iter().take(self.size) {
            other.insert(element);
        }
    }

    fn move_to_split(
        self,
        target: &Inode<T>,
        target_bounds: Rectangle,
        packed_lnodes: &mut PackedData<LeafNode<T>>,
    ) {
        let child_rects = target_bounds.get_children();
        for element in self.elements.into_iter().take(self.size) {
            for (i, &child_rect) in child_rects.iter().enumerate() {
                if child_rect.contains_point(element.position()) {
                    match target.get_child(i) {
                        Node::Leaf(leaf_node_id) => {
                            packed_lnodes.get_mut(leaf_node_id).insert(element);
                            break;
                        }
                        Node::Inner(_) => panic!("Cannot move elements to non-leaf target node."),
                    }
                }
            }
        }
    }
}

struct Element<T> {
    data: T,
    position: Vec2,
    hash: u64,
}

impl<T> Element<T> {
    fn new(data: T, position: Vec2, hash: u64) -> Self {
        Self {
            data,
            position,
            hash,
        }
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

    #[inline]
    fn matches_tree_item(&self, tree_item: &TreeItem<T>) -> bool {
        self.hash == tree_item.hash()
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
        // TODO
    }

    fn get_children(&self) -> [Self; 4] {
        let middle = self.min.lerp(self.max, 0.5);
        [
            Self {
                min: self.min,
                max: middle,
            },
            Self {
                min: Vec2::new(middle.x, self.min.y),
                max: Vec2::new(self.max.x, middle.y),
            },
            Self {
                min: Vec2::new(self.min.x, middle.y),
                max: Vec2::new(middle.x, self.max.y),
            },
            Self {
                min: middle,
                max: self.max,
            },
        ]
    }
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
    fn test_update_position() {
        let mut tree = Tree::with_capacity(10, Rectangle::new(Vec2::ZERO, Vec2::ONE));

        let mut item = tree.insert(1, Vec2::ZERO);
        assert_eq!(tree.within_disc(Disc::new(Vec2::ZERO, 0.5)).len(), 1);
        assert_eq!(tree.within_disc(Disc::new(Vec2::ONE, 0.5)).len(), 0);

        tree.update_position(&mut item, Vec2::ONE);
        assert_eq!(tree.within_disc(Disc::new(Vec2::ZERO, 0.5)).len(), 0);
        assert_eq!(tree.within_disc(Disc::new(Vec2::ONE, 0.5)).len(), 1);

        tree.update_position(&mut item, Vec2::ZERO);
        assert_eq!(tree.within_disc(Disc::new(Vec2::ZERO, 0.5)).len(), 1);
        assert_eq!(tree.within_disc(Disc::new(Vec2::ONE, 0.5)).len(), 0);
    }

    #[test]
    #[should_panic]
    fn test_tree_too_deep() {
        let mut tree: Tree<u32> =
            Tree::with_capacity(1000, Rectangle::new(Vec2::ZERO, Vec2::new(4., 4.)));

        for i in 0..100000 {
            let pos = i as f32 * 0.00000001;
            tree.insert(i, Vec2::new(pos, pos));
        }
    }

    #[test]
    fn test_tree_split() {
        let mut tree: Tree<u32> =
            Tree::with_capacity(1000, Rectangle::new(Vec2::ZERO, Vec2::new(4., 4.)));

        let down_left_leaf_id = match tree.packed_inodes.get(tree.root_inode_id).get_child(2) {
            Node::Leaf(leaf_id) => leaf_id,
            Node::Inner(_) => panic!("Expected only depth 1."),
        };

        tree.split(
            tree.root_inode_id,
            2,
            down_left_leaf_id,
            Rectangle::new(Vec2::new(0., 2.), Vec2::new(2., 4.)),
        );

        match tree.packed_inodes.get(tree.root_inode_id).get_child(2) {
            Node::Leaf(_) => panic!("Expected depth 2."),
            Node::Inner(inode_id) => {
                for child in tree.packed_inodes.get(inode_id).children() {
                    match child {
                        Node::Leaf(leaf_id) => assert_eq!(tree.packed_lnodes.get(leaf_id).size, 0),
                        Node::Inner(_) => panic!("Expected depth 2."),
                    }
                }
            }
        }

        match tree.packed_inodes.get(tree.root_inode_id).get_child(0) {
            Node::Leaf(leaf_id) => assert_eq!(tree.packed_lnodes.get(leaf_id).size, 0),
            Node::Inner(_) => panic!("Expected depth 1."),
        };
        match tree.packed_inodes.get(tree.root_inode_id).get_child(1) {
            Node::Leaf(leaf_id) => assert_eq!(tree.packed_lnodes.get(leaf_id).size, 0),
            Node::Inner(_) => panic!("Expected depth 1."),
        };
        match tree.packed_inodes.get(tree.root_inode_id).get_child(3) {
            Node::Leaf(leaf_id) => assert_eq!(tree.packed_lnodes.get(leaf_id).size, 0),
            Node::Inner(_) => panic!("Expected depth 1."),
        };
    }

    #[test]
    fn test_tree() {
        let mut tree = Tree::with_capacity(
            1000,
            Rectangle::new(Vec2::new(-100., -50.), Vec2::new(200., 100.)),
        );

        // Add a few points very close together.
        let mut almost_in_center_item = tree.insert(1, Vec2::new(0.0001, 0.0001));
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

        assert_eq!(tree.packed_inodes.len(), 2);
        assert_eq!(tree.packed_lnodes.len(), 7);

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
        tree.update_position(&mut almost_in_center_item, Vec2::new(21.2, 22.2));
        let neighbours = tree.within_disc(Disc::new(Vec2::ZERO, 1.));
        assert_eq!(neighbours.len(), 3);
        let neighbours = tree.within_disc(Disc::new(Vec2::new(21.21, 22.21), 0.1));
        assert_eq!(neighbours.len(), 1);

        // Test point removal.
        for tree_item in tree_items {
            tree.remove(tree_item);
        }
        assert_eq!(tree.packed_lnodes.len(), 4);
        assert_eq!(tree.packed_inodes.len(), 1);
        let neighbours = tree.within_disc(Disc::new(Vec2::ZERO, 1.));
        assert_eq!(neighbours.len(), 2);
    }

    #[test]
    fn test_leaf_node_move_to() {
        let mut leaf_node = LeafNode::empty();
        leaf_node.insert(Element::new(1, Vec2::new(1., 1.), 11));
        leaf_node.insert(Element::new(2, Vec2::new(7., 8.), 12));
        leaf_node.insert(Element::new(3, Vec2::new(17., 8.), 13));

        let mut packed_lnodes = PackedData::with_max_capacity(4);
        let target_node = Inode::with_empty_leafs(&mut packed_lnodes);
        let target_bounds = Rectangle::new(Vec2::new(0., 0.), Vec2::new(20., 20.));

        leaf_node.move_to_split(&target_node, target_bounds, &mut packed_lnodes);

        let left_up = match target_node.get_child(0) {
            Node::Leaf(leaf_node_id) => packed_lnodes.get(leaf_node_id),
            Node::Inner(_) => panic!("Expected leaf node."),
        };
        let right_up = match target_node.get_child(1) {
            Node::Leaf(leaf_node_id) => packed_lnodes.get(leaf_node_id),
            Node::Inner(_) => panic!("Expected leaf node."),
        };
        let left_down = match target_node.get_child(2) {
            Node::Leaf(leaf_node_id) => packed_lnodes.get(leaf_node_id),
            Node::Inner(_) => panic!("Expected leaf node."),
        };
        let right_down = match target_node.get_child(3) {
            Node::Leaf(leaf_node_id) => packed_lnodes.get(leaf_node_id),
            Node::Inner(_) => panic!("Expected leaf node."),
        };

        assert_eq!(left_up.size(), 2);
        assert_eq!(left_up.elements[0].data, 1);
        assert_eq!(left_up.elements[1].data, 2);
        assert_eq!(right_up.size(), 1);
        assert_eq!(right_up.elements[0].data, 3);
        assert_eq!(left_down.size(), 0);
        assert_eq!(right_down.size(), 0);
    }

    #[test]
    fn test_leaf_node() {
        let mut leaf_node = LeafNode::empty();
        leaf_node.insert(Element::new(1, Vec2::new(-100., -100.), 11));
        leaf_node.insert(Element::new(2, Vec2::new(7., 8.), 12));
        leaf_node.insert(Element::new(3, Vec2::new(17., 8.), 13));

        // Test disc not matching any elements
        let mut target = Vec::new();
        leaf_node.push_within_disc(Disc::new(Vec2::new(5., 4.), 4.), &mut target);
        assert_eq!(target.len(), 0);

        // Test disc matching only a single element
        let mut target = Vec::new();
        leaf_node.push_within_disc(Disc::new(Vec2::new(5., 4.), 5.), &mut target);
        assert_eq!(target.len(), 1);
        assert_eq!(*target[0].0, 2);
        assert_eq!(target[0].1, Vec2::new(7., 8.));

        // Test all encompassing disc
        let mut target = Vec::new();
        leaf_node.push_within_disc(Disc::new(Vec2::new(5., 4.), 1000.), &mut target);
        assert_eq!(target.len(), 3);
        assert_eq!(*target[0].0, 1);
        assert_eq!(target[0].1, Vec2::new(-100., -100.));
        assert_eq!(*target[1].0, 2);
        assert_eq!(target[1].1, Vec2::new(7., 8.));
        assert_eq!(*target[2].0, 3);
        assert_eq!(target[2].1, Vec2::new(17., 8.));

        // Test removal of an element
        leaf_node.remove(leaf_node.find(&TreeItem::new(12, Vec2::ZERO)));
        let mut target = Vec::new();
        leaf_node.push_within_disc(Disc::new(Vec2::new(5., 4.), 1000.), &mut target);
        assert_eq!(target.len(), 2);
        assert_eq!(*target[0].0, 1);
        assert_eq!(target[0].1, Vec2::new(-100., -100.));
        assert_eq!(*target[1].0, 3);
        assert_eq!(target[1].1, Vec2::new(17., 8.));

        // Test empty node after removal of all nodes
        leaf_node.remove(leaf_node.find(&TreeItem::new(11, Vec2::ZERO)));
        leaf_node.remove(leaf_node.find(&TreeItem::new(13, Vec2::ZERO)));
        let mut target = Vec::new();
        leaf_node.push_within_disc(Disc::new(Vec2::new(5., 4.), 1000.), &mut target);
        assert_eq!(target.len(), 0);
    }

    #[test]
    fn test_element() {
        let element = Element::new(1, Vec2::new(7., 8.), 77);
        assert!(element.is_within_disc(Disc::new(Vec2::new(7., 8.), 0.0001)));
        assert!(element.is_within_disc(Disc::new(Vec2::new(6., 9.), 1.5)));
        assert!(!element.is_within_disc(Disc::new(Vec2::new(6., 9.), 1.)));
    }

    #[test]
    fn test_rectangle() {
        let rectangle = Rectangle::new(Vec2::new(1., 2.), Vec2::new(5., 3.));

        let [left_up, right_up, left_down, right_down] = rectangle.get_children();
        assert_eq!(
            left_up,
            Rectangle::new(Vec2::new(1., 2.), Vec2::new(3., 2.5))
        );
        assert_eq!(
            right_up,
            Rectangle::new(Vec2::new(3., 2.), Vec2::new(5., 2.5))
        );
        assert_eq!(
            left_down,
            Rectangle::new(Vec2::new(1., 2.5), Vec2::new(3., 3.))
        );
        assert_eq!(
            right_down,
            Rectangle::new(Vec2::new(3., 2.5), Vec2::new(5., 3.))
        );

        assert!(rectangle.intersects_disc(Disc::new(Vec2::new(1.5, 2.5), 0.001)));
        assert!(rectangle.intersects_disc(Disc::new(Vec2::new(-5., -9.), 13.)));
        assert!(!rectangle.intersects_disc(Disc::new(Vec2::new(-5., -9.), 12.)));
    }

    #[test]
    fn test_disc() {
        let disc = Disc::new(Vec2::new(2.5, 5.5), 6.);
        assert_eq!(disc.center(), Vec2::new(2.5, 5.5));
        assert_eq!(disc.radius_squared(), 36.);
    }
}
