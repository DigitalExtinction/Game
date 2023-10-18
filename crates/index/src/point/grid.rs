use std::num::NonZeroU16;

use glam::{IVec2, Vec2};

struct Grid {
    half_size: Vec2,
    stems: Vec<Stem>,
    leafs: Vec<Leaf>,
}

impl Grid {
    fn add(&mut self, point: Vec2, id: u16) {
        let mut link = Link::Stem(0);
        let mut min = -self.half_size;
        let mut max = self.half_size;

        loop {
            match link {
                Link::Stem(index) => {
                    let stem = &self.stems[index];

                    let mid = (max + min) / 2.;
                    let edge = if mid.x <= point.x {
                        min.x = mid.x;
                        1
                    } else {
                        max.x = mid.x;
                        0
                    } + if mid.y <= point.y {
                        min.y = mid.y;
                        2
                    } else {
                        max.y = mid.y;
                        0
                    };

                    match stem.edges[edge].to_link() {
                        None => {
                            debug_assert!(self.leafs.len() <= (u16::MAX as usize));
                            let leaf = self.leafs.len();
                            self.leafs.push(Leaf::new());
                            self.stems[index].edges[edge] = CompactEdge::leaf(leaf);
                            link = Link::Leaf(leaf);
                        }
                        Some(next_link @ Link::Stem(_)) => {
                            link = next_link;
                        }
                        Some(Link::Leaf(leaf)) => {
                            // TODO if full split

                            link = Link::Leaf(leaf);
                        }
                    }
                }
                Link::Leaf(index) => {
                    self.leafs[index].push(id, point);
                }
            }
        }
    }
}

struct Stem {
    size: u16,
    edges: [CompactEdge; 4],
}

struct CompactEdge(u16);

impl CompactEdge {
    const SPLIT_BIT: u16 = 0b1000_0000_0000_0000;
    const BIT_MASK: u16 = Self::SPLIT_BIT - 1;

    fn empty() -> Self {
        Self(u16::MAX)
    }

    fn leaf(leaf: usize) -> Self {
        debug_assert!(leaf < Self::BIT_MASK as usize);
        Self(leaf as u16 + Self::SPLIT_BIT)
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
    ids: [Item; 16],
}

impl Leaf {
    fn new() -> Self {
        Self {
            size: 0,
            ids: [Item::default(); 16],
        }
    }

    fn push(&mut self, id: u16, point: Vec2) {
        debug_assert!((self.size as usize) < self.ids.len());
        self.ids[self.size as usize] = Item { id, point };
        self.size += 1;
    }
}

#[derive(Default, Clone, Copy)]
struct Item {
    id: u16,
    point: Vec2,
}
