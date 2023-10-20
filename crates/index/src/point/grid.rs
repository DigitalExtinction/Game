use glam::{IVec2, Vec2};

use super::subdivision::{self, Rectangle, Subdivisions};

struct Grid {
    half_size: Vec2,
    stems: Vec<Stem>,
    leafs: Vec<Leaf>,
}

impl Grid {
    fn add(&mut self, point: Vec2, id: u16) {
        let mut link = Link::Stem(0);
        let mut subdivisions = Subdivisions::new(Rectangle::from_half_size(self.half_size), point);

        for (rectangle, sector) in subdivisions {
            match link {
                Link::Stem(index) => match self.stems[index].children[sector.index()].to_link() {
                    None => {
                        let leaf = self.leafs.len();
                        self.leafs.push(Leaf::new());
                        self.stems[index].children[sector.index()] = CompactEdge::leaf(leaf);
                        link = Link::Leaf(leaf);
                    }
                    Some(next_link @ Link::Stem(_)) => {
                        link = next_link;
                    }
                    Some(Link::Leaf(leaf)) => {
                        if self.leafs[leaf].is_full() {
                            let new_stem_index = self.stems.len();
                            let mut new_stem = Stem::default();

                            // TODO
                            self.leafs[leaf].x(leaf, &mut new_stem);

                            self.stems.push(new_stem);
                            self.stems[index].children[sector.index()] =
                                CompactEdge::stem(new_stem_index);
                        } else {
                            link = Link::Leaf(leaf);
                        }
                    }
                },
                Link::Leaf(index) => {
                    self.leafs[index].push(id, point);
                }
            }
        }
    }
}

struct Stem {
    children: [CompactEdge; 4],
}

impl Default for Stem {
    fn default() -> Self {
        Self {
            children: [
                CompactEdge::empty(),
                CompactEdge::empty(),
                CompactEdge::empty(),
                CompactEdge::empty(),
            ],
        }
    }
}

struct CompactEdge(u16);

impl CompactEdge {
    const SPLIT_BIT: u16 = 1 << (u16::BITS - 1);
    const BIT_MASK: u16 = Self::SPLIT_BIT - 1;

    fn empty() -> Self {
        Self(u16::MAX)
    }

    fn leaf(leaf: usize) -> Self {
        debug_assert!(leaf < Self::BIT_MASK as usize);
        Self(leaf as u16 + Self::SPLIT_BIT)
    }

    fn stem(stem: usize) -> Self {
        debug_assert!(stem <= Self::BIT_MASK as usize);
        Self(stem as u16)
    }

    fn to_link(&self) -> Option<Link> {
        if self.0 == u16::MAX {
            None
        } else if self.0 < Self::SPLIT_BIT {
            Some(Link::Stem(self.0 as usize))
        } else {
            Some(Link::Leaf((self.0 & Self::BIT_MASK) as usize))
        }
    }
}

enum Link {
    Stem(usize),
    Leaf(usize),
}

struct Leaf {
    size: u8,
    items: [Item; 16],
}

impl Leaf {
    fn new() -> Self {
        Self {
            size: 0,
            items: [Item::default(); 16],
        }
    }

    fn is_full(&self) -> bool {
        (self.size as usize) >= self.items.len()
    }

    fn push(&mut self, id: u16, point: Vec2) {
        debug_assert!((self.size as usize) < self.items.len());
        self.items[self.size as usize] = Item { id, point };
        self.size += 1;
    }

    // TODO docs + panics
    fn x(&mut self, self_index: usize, stem: &mut Stem) {
        debug_assert!(self.size as usize == self.items.len());

        // TODO
    }
}

#[derive(Default, Clone, Copy)]
struct Item {
    id: u16,
    point: Vec2,
}
