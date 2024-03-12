use glam::Vec2;
use quadrants::{Quadrants, Rect};

mod quadrants;

pub struct Tree<T>
where
    T: Copy + Clone + Default + PartialEq,
{
    rect: Rect,
    inner: Vec<Inner>,
    leafs: Vec<Leaf<T>>,
}

impl<T> Tree<T>
where
    T: Copy + Clone + Default + PartialEq,
{
    pub fn insert(&mut self, pos: Vec2, item: T) {
        let item = Item { pos, item };

        let mut rect = self.rect.clone();
        let mut current = Slot::Inner(0);

        let target = loop {
            match current {
                Slot::Inner(index) => {
                    let quadrant = rect.quadrant(item.pos);
                    match self.inner[index].children.get(quadrant) {
                        Some(slot) => {
                            rect = rect.child(quadrant);
                            current = *slot;
                        }
                        None => {
                            current = Slot::Leaf(self.leafs.len());
                            self.leafs.push(Leaf::new(index));
                            self.inner[index].children.set(quadrant, Some(current));
                        }
                    }
                }
                Slot::Leaf(index) => {
                    if self.leafs[index].is_full() {
                        current = Slot::Inner(self.split(index, &rect));
                    } else {
                        break index;
                    }
                }
            }
        };

        self.leafs[target].insert(item);
    }

    pub fn remove(&mut self, pos: Vec2, item: T) {
        let item = Item { pos, item };

        let mut rect = self.rect.clone();
        let mut current = Slot::Inner(0);

        let target = loop {
            match current {
                Slot::Inner(index) => {
                    let quadrant = rect.quadrant(item.pos);
                    match self.inner[index].children.get(quadrant) {
                        Some(slot) => {
                            rect = rect.child(quadrant);
                            current = *slot;
                        }
                        None => {
                            // TODO point does not exist
                        }
                    }
                }
                Slot::Leaf(index) => {
                    break index;
                }
            }
        };

        let leaf = &mut self.leafs[target];
        let mut parent = leaf.parent;

        // TODO return the bool
        leaf.remove(item);

        if leaf.len == 0 {
            self.remove_leaf(target, None);
        }

        while self.mergable(parent) {
            let leaf_index = self.merge(parent);
            parent = self.leafs[leaf_index].parent;
        }
    }

    fn split(&mut self, index: usize, rect: &Rect) -> usize {
        let inner_index = self.inner.len();
        let removed = self.remove_leaf(index, Some(Slot::Inner(inner_index)));

        let mut leafs = Quadrants::new(
            Some(Leaf::new(inner_index)),
            Some(Leaf::new(inner_index)),
            Some(Leaf::new(inner_index)),
            Some(Leaf::new(inner_index)),
        );

        for item in removed.items.into_iter().take(removed.len) {
            leafs.get_mut(rect.quadrant(item.pos)).unwrap().insert(item);
        }

        let all_leafs = &mut self.leafs;
        let new_inner = Inner::new(
            Some(removed.parent),
            leafs.map(move |leaf| {
                let slot = Slot::Leaf(all_leafs.len());
                all_leafs.push(leaf);
                slot
            }),
        );

        self.inner.push(new_inner);
        inner_index
    }

    fn mergable(&self, index: usize) -> bool {
        if index == 0 {
            return false;
        }

        // TODO use a constant
        self.num_children(index).map_or(false, |num| num <= 8)
    }

    fn num_children(&self, index: usize) -> Option<usize> {
        let mut len = 0;
        for slot in &self.inner[index].children {
            match slot {
                Slot::Inner(_) => {
                    return None;
                }
                Slot::Leaf(child_index) => {
                    len += self.leafs[*child_index].len;
                }
            }
        }

        Some(len)
    }

    fn merge(&mut self, index: usize) -> usize {
        if index == 0 {
            panic!("Cannot merge root node.");
        }

        let parent = self.inner[index].parent.unwrap();
        let mut leaf = Leaf::new(parent);

        let mut num_indices = 0;
        let mut indices = [0; 4];

        for slot in &self.inner[index].children {
            match slot {
                Slot::Inner(_) => panic!("Cannot merge node with non-leaf children."),
                Slot::Leaf(index) => {
                    indices[num_indices] = *index;
                    num_indices += 1;
                }
            }
        }

        for index in indices.iter().take(num_indices) {
            let child = self.remove_leaf(*index, None);
            for item in child.items {
                leaf.items[leaf.len] = item;
                leaf.len += 1;
            }
        }

        self.remove_inner(index);
        self.inner[parent].replace_child(Slot::Inner(index), Some(Slot::Leaf(self.leafs.len())));

        let leaf_index = self.leafs.len();
        self.leafs.push(leaf);
        leaf_index
    }

    fn remove_inner(&mut self, index: usize) -> Inner {
        if index == 0 {
            panic!("Cannot remove root node.");
        }

        let removed = self.inner.swap_remove(index);

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
    children: Quadrants<Slot>,
}

impl Inner {
    fn new(parent: Option<usize>, children: Quadrants<Slot>) -> Self {
        Self { parent, children }
    }

    fn replace_child(&mut self, old: Slot, new: Option<Slot>) {
        self.children.replace(&old, new);
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
    T: Copy + Clone + Default + PartialEq,
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
    T: Copy + Clone + Default + PartialEq,
{
    fn new(parent: usize) -> Self {
        Self {
            parent,
            items: [Item::default(); 8],
            len: 0,
        }
    }

    fn is_full(&self) -> bool {
        self.len >= self.items.len()
    }

    fn insert(&mut self, item: Item<T>) {
        if self.len >= self.items.len() {
            panic!("Leaf is full.");
        }

        // TODO check for collision?

        self.items[self.len] = item;
        self.len += 1;
    }

    fn remove(&mut self, item: Item<T>) -> bool {
        for i in 0..self.len {
            if self.items[i] == item {
                self.len -= 1;
                // Move the item to the position of the first non-occupied slot
                // (swap remove).
                if i < self.len {
                    self.items[i] = self.items[self.len];
                }

                return true;
            }
        }

        false
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
struct Item<T>
where
    T: Clone + Copy + PartialEq,
{
    pos: Vec2,
    item: T,
}
