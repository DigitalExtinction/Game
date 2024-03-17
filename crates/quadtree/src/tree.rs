use crate::{
    packed::Packed,
    quadrants::{Quadrant, Quadrants},
};

pub(super) struct Tree<S>
where
    S: Default,
{
    inner: Packed<Inner>,
    leafs: Packed<Leaf<S>>,
}

impl<S> Tree<S>
where
    S: Default,
{
    pub(super) fn new() -> Self {
        let mut inner = Packed::new();
        // Add empty root.
        inner.push(Inner::new(usize::MAX));
        Self {
            inner,
            leafs: Packed::new(),
        }
    }

    pub(super) fn get_leaf(&self, index: usize) -> Option<&S> {
        self.leafs.get(index).map(|l| &l.item)
    }

    pub(super) fn get_leaf_mut(&mut self, index: usize) -> Option<&mut S> {
        self.leafs.get_mut(index).map(|l| &mut l.item)
    }

    pub(super) fn children(&self, index: usize) -> &Quadrants<Node> {
        &self.inner.get(index).unwrap().children
    }

    pub(super) fn remove_children(&mut self, index: usize) -> Quadrants<S> {
        let mut removed = Quadrants::empty();

        for quadrant in [
            Quadrant::TopLeft,
            Quadrant::TopRight,
            Quadrant::BottomLeft,
            Quadrant::BottomRight,
        ] {
            if let Some(&child) = self.inner.get(index).unwrap().children.get(quadrant) {
                match child {
                    Node::Inner(_) => {
                        panic!("Cannot remove non-leaf children from a node.");
                    }
                    Node::Leaf(child_index) => {
                        removed.set(quadrant, Some(self.remove_leaf(child_index)));
                    }
                }
            }
        }

        removed
    }

    pub(super) fn remove_inner(&mut self, index: usize) {
        self.replace_internal(Node::Inner(index), None);
    }

    pub(super) fn remove_leaf(&mut self, index: usize) -> S {
        self.replace_internal(Node::Leaf(index), None).unwrap()
    }

    pub(super) fn replace_inner(&mut self, index: usize) -> usize {
        let parent = self.inner.get(index).unwrap().parent;
        let new_leaf_index = self.leafs.len();

        self.leafs.push(Leaf::new(parent));
        self.replace_internal(Node::Inner(index), Some(Node::Leaf(new_leaf_index)));

        new_leaf_index
    }

    pub(super) fn replace_leaf(&mut self, index: usize) -> (usize, S) {
        let parent = self.leafs.get(index).unwrap().parent;
        let new_inner_index = self.inner.len();

        self.inner.push(Inner::new(parent));
        let item = self
            .replace_internal(Node::Leaf(index), Some(Node::Inner(new_inner_index)))
            .unwrap();

        (new_inner_index, item)
    }

    fn replace_internal(&mut self, node: Node, replacement: Option<Node>) -> Option<S> {
        let (removed_parent, moved, item) = match node {
            Node::Inner(index) => {
                if index == 0 {
                    panic!("Cannot remove or replace root node.");
                }

                let (removed, moved) = self.inner.swap_remove(index);
                (removed.parent, moved.map(|index| Node::Inner(index)), None)
            }
            Node::Leaf(index) => {
                let (removed, moved) = self.leafs.swap_remove(index);
                (
                    removed.parent,
                    moved.map(|index| Node::Leaf(index)),
                    Some(removed.item),
                )
            }
        };

        self.inner
            .get_mut(removed_parent)
            .unwrap()
            .replace_child(node, replacement);

        if let Some(moved) = moved {
            let moved_parent = match node {
                Node::Inner(index) => self.inner.get(index).unwrap().parent,
                Node::Leaf(index) => self.leafs.get(index).unwrap().parent,
            };

            self.inner
                .get_mut(moved_parent)
                .unwrap()
                .replace_child(moved, Some(node));
        }

        item
    }
}

// TODO rename to Node
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Node {
    // TODO consider compressing usize to something smaller & saving the extra
    // byte for enum
    Inner(usize),
    Leaf(usize),
}

struct Inner {
    // TODO consider using MAX value for no parent
    // TODO consider using something smaller than usize
    parent: usize,
    children: Quadrants<Node>,
}

impl Inner {
    fn new(parent: usize) -> Self {
        Self {
            parent,
            children: Quadrants::empty(),
        }
    }

    fn replace_child(&mut self, old: Node, new: Option<Node>) {
        self.children.replace(&old, new);
    }
}

struct Leaf<S> {
    // TODO consider using something smaller than usize
    parent: usize,
    // TODO consider using different array len
    item: S,
}

impl<S> Leaf<S>
where
    S: Default,
{
    fn new(parent: usize) -> Self {
        Self {
            parent,
            item: S::default(),
        }
    }
}
