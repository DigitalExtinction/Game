use crate::{packed::Packed, quadrants::Quadrants};

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
        inner.push(Inner::new(usize::MAX, Quadrants::default()));
        Self {
            inner,
            leafs: Packed::new(),
        }
    }

    pub(super) fn get_leaf_mut(&self, index: usize) -> Option<&mut S> {
        self.leafs.get_mut(index).map(|l| &mut l.item)
    }

    // TODO remove
    // TODO replace (auto crate the replacement)

    fn replace(&mut self, node: Node, replacement: Option<Node>) -> Option<S> {
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
enum Node {
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
    fn new(parent: usize, children: Quadrants<Node>) -> Self {
        Self { parent, children }
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
