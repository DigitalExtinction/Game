struct Tree<T>
where
    T: Default,
{
    min: [f32; 2],
    max: [f32; 2],
    inner: Vec<Inner>,
    leafs: Vec<Leaf<T>>,
}

impl<T> Tree<T>
where
    T: Default,
{
    fn insert(&mut self, pos: [f32; 2], item: T) {
        // TODO locate target leaf
        // TODO if it doesn't exist: create it
        // TODO if it is full: recursively split
        // TODO insert the item
        // TODO check for collision
    }

    fn remove(&mut self, pos: [f32; 2]) {
        // TODO locate target leaf
        // TODO remove the item from the leaf
        // TODO merge to parent if:
        //  * parent has a parent (i.e. parent is not root)
        //  * all children fit into a single leaf
        //  * parent has only leafs or empty children slots
    }

    fn merge(&mut self, index: usize) {
        // TODO update index of child

        // let Some(parent) = self.parent else {
        //     panic!("Cannot merge root node.");
        // };

        // let mut merged = Leaf::new(parent);
        // for i in 0..4 {
        //     match self.children[i] {
        //         Slot::Inner(_) => panic!("Cannot merge node with non-leaf children."),
        //         Slot::Leaf(index) => {
        //             // TODO
        //         }
        //         Slot::Empty => (),
        //     }
        // }
        // merged
    }

    fn remove_inner(&mut self, index: usize) -> Inner {
        if index == 0 {
            panic!("Cannot remove root node.");
        }

        let removed = self.inner.swap_remove(index);

        let old_index = self.inner.len();
        if index != old_index {
            if let Some(parent) = self.inner[index].parent {
                self.inner[parent].replace_child(Slot::Inner(old_index), Slot::Inner(index));
            }
        }

        removed
    }

    fn remove_leaf(&mut self, index: usize) -> Leaf<T> {
        let removed = self.leafs.swap_remove(index);

        let old_index = self.leafs.len();
        if index != old_index {
            let parent = self.leafs[index].parent;
            self.inner[parent].replace_child(Slot::Leaf(old_index), Slot::Leaf(index));
        }

        removed
    }
}

struct Inner {
    // TODO consider using MAX value for no parent
    // TODO consider using something smaller than usize
    parent: Option<usize>,
    children: [Slot; 4],
}

impl Inner {
    fn new(parent: Option<usize>) -> Self {
        Self {
            parent,
            children: [Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty],
        }
    }

    fn replace_child(&mut self, old: Slot, new: Slot) {
        for i in 0..4 {
            let target = &mut self.children[i];
            if *target == old {
                *target = new;
                return;
            }
        }

        panic!("No child moved.");
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Slot {
    // TODO consider compressing usize to something smaller & saving the extra
    // byte for enum
    Inner(usize),
    Leaf(usize),
    Empty,
}

struct Leaf<T>
where
    T: Default,
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
    T: Default,
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

    fn insert(&mut self, pos: [f32; 2], item: T) {
        if self.len >= self.items.len() {
            panic!("Leaf is full.");
        }

        self.items[self.len] = Item { pos, item };
        self.len += 1;
    }

    fn remove(&mut self, pos: [f32; 2]) -> Option<T> {
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
    T: Default,
{
    pos: [f32; 2],
    item: T,
}
