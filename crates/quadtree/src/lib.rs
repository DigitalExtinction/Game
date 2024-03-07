use glam::Vec2;
use quadrants::Rect;

mod quadrants;

struct Tree<T>
where
    T: Default,
{
    rect: Rect,
    inner: Vec<Inner>,
    leafs: Vec<Leaf<T>>,
}

impl<T> Tree<T>
where
    T: Sized + Default,
{
    fn insert(&mut self, pos: [f32; 2], item: T) {
        // TODO locate target leaf
        // TODO if it doesn't exist: create it
        // TODO if it is full: recursively split
        // TODO insert the item
        // TODO check for collision

        let rect = self.rect.clone();
        let mut current = Slot::Inner(0);

        let target = loop {
            match current {
                Slot::Inner(index) => {
                    // TODO set current to proper child
                }
                Slot::Leaf(index) => {
                    if self.leafs[index].is_full() {
                        let innder_index = self.split(index, &rect);
                        current = Slot::Inner(innder_index);
                    } else {
                        break index;
                    }
                }
            }
        };
    }

    fn remove(&mut self, pos: [f32; 2]) {
        // TODO locate target leaf
        // TODO remove the item from the leaf
        // TODO merge to parent if:
        //  * parent has a parent (i.e. parent is not root)
        //  * all children fit into a single leaf
        //  * parent has only leafs or empty children slots
    }

    fn split(&mut self, index: usize, rect: &Rect) -> usize {
        let inner_index = self.inner.len();
        let removed = self.remove_leaf(index, Some(Slot::Inner(inner_index)));

        let mut new_inner = Inner::new(Some(removed.parent));
        let mut leafs = [
            Leaf::new(inner_index),
            Leaf::new(inner_index),
            Leaf::new(inner_index),
            Leaf::new(inner_index),
        ];

        for item in removed.items.into_iter().take(removed.len) {
            leafs[rect.quadrant(item.pos)].insert(item.pos, item.item);
        }

        for (i, leaf) in leafs.into_iter().enumerate() {
            new_inner.children[i] = Some(Slot::Leaf(self.leafs.len()));
            self.leafs.push(leaf);
        }

        self.inner.push(new_inner);
        inner_index
    }

    fn merge(&mut self, index: usize) {
        if index == 0 {
            panic!("Cannot merge root node.");
        }

        let parent = self.inner[index].parent.unwrap();
        let mut leaf = Leaf::new(parent);

        for slot in self.inner[index].children {
            if let Some(child) = slot {
                match child {
                    Slot::Inner(_) => panic!("Cannot merge node with non-leaf children."),
                    Slot::Leaf(index) => {
                        let child = self.remove_leaf(index, None);
                        for item in child.items {
                            leaf.items[leaf.len] = item;
                            leaf.len += 1;
                        }
                    }
                }
            }
        }

        self.remove_inner(index);
        self.inner[parent].replace_child(Slot::Inner(index), Some(Slot::Leaf(self.leafs.len())));
        self.leafs.push(leaf);
    }

    fn remove_inner(&mut self, index: usize) -> Inner {
        if index == 0 {
            panic!("Cannot remove root node.");
        }

        let removed = self.inner.swap_remove(index);

        // TODO: only in debug mode
        for i in 0..4 {
            if !matches!(removed.children[0], None) {
                panic!("Cannot remove non-empty inner node.");
            }
        }

        let old_index = self.inner.len();
        if index != old_index {
            if let Some(parent) = self.inner[index].parent {
                self.inner[parent].replace_child(Slot::Inner(old_index), Some(Slot::Inner(index)));
            }
        }

        removed
    }

    fn remove_leaf(&mut self, index: usize, replacement: Option<Slot>) -> Leaf<T> {
        let removed = self.leafs.swap_remove(index);

        self.inner[removed.parent].replace_child(Slot::Leaf(index), replacement);

        let old_index = self.leafs.len();
        if index != old_index {
            let parent = self.leafs[index].parent;
            self.inner[parent].replace_child(Slot::Leaf(old_index), Some(Slot::Leaf(index)));
        }

        removed
    }
}

struct Inner {
    // TODO consider using MAX value for no parent
    // TODO consider using something smaller than usize
    parent: Option<usize>,
    children: [Option<Slot>; 4],
}

impl Inner {
    fn new(parent: Option<usize>) -> Self {
        Self {
            parent,
            children: [None; 4],
        }
    }

    fn replace_child(&mut self, old: Slot, new: Option<Slot>) {
        for target in &mut self.children {
            if target.map_or(false, |t| t == old) {
                *target = new;
                return;
            }
        }

        panic!("No child moved.");
    }
}

// TODO rename to Node
#[derive(Clone, Copy, PartialEq, Eq)]
enum Slot {
    // TODO consider compressing usize to something smaller & saving the extra
    // byte for enum
    Inner(usize),
    Leaf(usize),
}

struct Leaf<T>
where
    T: Sized + Default,
{
    // TODO consider using something smaller than usize
    parent: usize,
    // TODO consider using different array len
    items: [Item<T>; 8],
    // TODO consider using something smaller than usize
    len: usize,
}

impl<T> Leaf<T>
where
    T: Sized + Default,
{
    fn new(parent: usize) -> Self {
        Self {
            parent,
            items: [
                Item::default(),
                Item::default(),
                Item::default(),
                Item::default(),
                Item::default(),
                Item::default(),
                Item::default(),
                Item::default(),
            ],
            len: 0,
        }
    }

    fn is_full(&self) -> bool {
        self.len >= self.items.len()
    }

    fn insert(&mut self, pos: Vec2, item: T) {
        if self.len >= self.items.len() {
            panic!("Leaf is full.");
        }

        self.items[self.len] = Item { pos, item };
        self.len += 1;
    }

    fn remove(&mut self, pos: Vec2) -> Option<T> {
        for i in 0..self.len {
            if pos == self.items[i].pos {
                self.len -= 1;

                // First move the item to the position of the last occupied
                // slot as part of the swap remove.
                if i < self.len {
                    unsafe {
                        std::ptr::swap(&mut self.items[self.len], &mut self.items[i]);
                    }
                }

                // Then move it out (replace it but a placeholder).
                let mut item = Item::default();
                std::mem::swap(&mut self.items[self.len], &mut item);

                return Some(item.item);
            }
        }

        None
    }
}

#[derive(Default)]
struct Item<T>
where
    T: Sized + Default,
{
    pos: Vec2,
    item: T,
}
