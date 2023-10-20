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

        for child in Path::from_half_size(self.half_size, point) {
            match link {
                Link::Stem(index) => {
                    let stem = &self.stems[index];

                    match stem.children[child].to_link() {
                        None => {
                            let leaf = self.leafs.len();
                            self.leafs.push(Leaf::new());
                            self.stems[index].children[child] = CompactEdge::leaf(leaf);
                            link = Link::Leaf(leaf);
                        }
                        Some(next_link @ Link::Stem(_)) => {
                            link = next_link;
                        }
                        Some(Link::Leaf(leaf)) => {
                            if self.leafs[leaf].is_full() {
                                // TODO
                            } else {
                                link = Link::Leaf(leaf);
                            }
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

// TODO docs (min_x, min_y, max_x, min_y, ...)
// TODO tests
/// Each rectangle in the quad-tree has four children. This is the index of the
/// child (of the last traversed parent) corresponding the current
/// rectangle.
struct Path {
    target: Vec2,
    min: Vec2,
    max: Vec2,
}

impl Path {
    fn from_half_size(half_size: Vec2, target: Vec2) -> Self {
        Self {
            target,
            min: -half_size,
            max: half_size,
        }
    }
}

impl Iterator for Path {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let mid = self.max.lerp(self.min, 0.5);

        let x_offset = if mid.x <= self.target.x {
            self.min.x = mid.x;
            1
        } else {
            self.max.x = mid.x;
            0
        };

        let y_offset = if mid.y <= self.target.y {
            self.min.y = mid.y;
            2
        } else {
            self.max.y = mid.y;
            0
        };

        Some(x_offset + y_offset)
    }
}

struct Stem {
    size: u16,
    children: [CompactEdge; 4],
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
    ids: [Item; 16],
}

impl Leaf {
    fn new() -> Self {
        Self {
            size: 0,
            ids: [Item::default(); 16],
        }
    }

    fn is_full(&self) -> bool {
        (self.size as usize) >= self.ids.len()
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
