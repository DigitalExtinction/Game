use glam::Vec2;
use quadrants::{Quadrants, Rect};
use tree::{Node, Tree};

mod packed;
mod quadrants;
mod tree;

pub struct TreeX<T>
where
    T: Copy + Clone + Default + PartialEq,
{
    rect: Rect,
    tree: Tree<Items<T>>,
}

impl<T> TreeX<T>
where
    T: Copy + Clone + Default + PartialEq,
{
    // pub fn insert(&mut self, pos: Vec2, item: T) {
    //     let item = Item { pos, item };

    //     let mut rect = self.rect.clone();
    //     let mut current = Slot::Inner(0);

    //     let target = loop {
    //         match current {
    //             Slot::Inner(index) => {
    //                 let quadrant = rect.quadrant(item.pos);
    //                 match self.inner[index].children.get(quadrant) {
    //                     Some(slot) => {
    //                         rect = rect.child(quadrant);
    //                         current = *slot;
    //                     }
    //                     None => {
    //                         current = Slot::Leaf(self.leafs.len());
    //                         self.leafs.push(Leaf::new(index));
    //                         self.inner[index].children.set(quadrant, Some(current));
    //                     }
    //                 }
    //             }
    //             Slot::Leaf(index) => {
    //                 if self.leafs[index].is_full() {
    //                     current = Slot::Inner(self.split(index, &rect));
    //                 } else {
    //                     break index;
    //                 }
    //             }
    //         }
    //     };

    //     self.leafs[target].insert(item);
    // }

    pub fn remove(&mut self, pos: Vec2, item: T) {
        let item = Item { pos, item };

        let mut rect = self.rect.clone();
        let mut current = Tree::<Items<T>>::ROOT;

        let target = loop {
            match current {
                Node::Inner(index) => {
                    let quadrant = rect.quadrant(item.pos);
                    match self.tree.children(index).get(quadrant) {
                        Some(slot) => {
                            rect = rect.child(quadrant);
                            current = *slot;
                        }
                        None => {
                            // TODO point does not exist
                        }
                    }
                }
                Node::Leaf(index) => {
                    break index;
                }
            }
        };

        let items = self.tree.get_leaf_mut(target).unwrap();
        if !items.remove(item) {
            // TODO point does not exist
        }
        let is_empty = items.len == 0;

        let mut parent = self.tree.leaf_parent(target);
        if is_empty {
            self.tree.remove_leaf(target);
        }

        while self.mergable(parent) {
            let leaf_index = self.merge(parent);
            parent = self.tree.leaf_parent(leaf_index);
        }
    }

    fn split(&mut self, index: usize, rect: &Rect) -> usize {
        let (new_inner_index, items) = self.tree.replace_leaf(index);
        for item in items.items.into_iter().take(items.len) {
            self.insert(new_inner_index, rect, item);
        }
        new_inner_index
    }

    fn insert(&mut self, index: usize, rect: &Rect, item: Item<T>) {
        let quadrant = rect.quadrant(item.pos);
        let node = self.tree.init_child(index, quadrant, Items::new);
        let items = match node {
            Node::Inner(_) => panic!("Cannot insert item to an inner node."),
            Node::Leaf(index) => self.tree.get_leaf_mut(index).unwrap(),
        };
        items.insert(item);
    }

    fn mergable(&self, index: usize) -> bool {
        if index == 0 {
            return false;
        }

        // TODO use a constant (half of max size)
        self.num_children(index).map_or(false, |num| num <= 8)
    }

    fn num_children(&self, index: usize) -> Option<usize> {
        let mut len = 0;

        for &child in self.tree.children(index) {
            match child {
                Node::Inner(_) => {
                    return None;
                }
                Node::Leaf(child_index) => {
                    len += self.tree.get_leaf(child_index).unwrap().len;
                }
            }
        }

        Some(len)
    }

    fn merge(&mut self, index: usize) -> usize {
        if index == 0 {
            panic!("Cannot merge root node.");
        }

        let removed = self.tree.remove_children(index);
        let new_leaf_index = self.tree.replace_inner(index, Items::new());

        let leaf = self.tree.get_leaf_mut(new_leaf_index).unwrap();

        for items in &removed {
            for item in items.items.into_iter().take(items.len) {
                leaf.items[leaf.len] = item;
                leaf.len += 1;
            }
        }

        new_leaf_index
    }
}

// TODO
#[derive(Default)]
struct Items<T>
where
    T: Copy + Clone + Default + PartialEq,
{
    // TODO consider using something smaller than usize
    len: usize,
    // TODO consider using different array len
    items: [Item<T>; 8],
}

impl<T> Items<T>
where
    T: Copy + Clone + Default + PartialEq,
{
    fn new() -> Self {
        Self {
            len: 0,
            items: [Item::default(); 8],
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
