use std::hash::Hash;

use bevy::core::FloatOrd;
use parry2d::shape::Segment;

/// Line segment whose hash and equivalence class don't change with
/// orientation.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub(crate) struct HashableSegment((FloatOrd, FloatOrd), (FloatOrd, FloatOrd));

impl HashableSegment {
    pub(crate) fn new(segment: Segment) -> Self {
        let a = (FloatOrd(segment.a.x), FloatOrd(segment.a.y));
        let b = (FloatOrd(segment.b.x), FloatOrd(segment.b.y));
        if a < b {
            Self(a, b)
        } else {
            Self(b, a)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    use parry2d::math::Point;

    use super::*;

    #[test]
    fn test_hashable_segment() {
        let a = hash(HashableSegment::new(Segment::new(
            Point::new(1., 2.),
            Point::new(3., 4.),
        )));
        let b = hash(HashableSegment::new(Segment::new(
            Point::new(3., 4.),
            Point::new(1., 2.),
        )));
        let c = hash(HashableSegment::new(Segment::new(
            Point::new(2., 1.),
            Point::new(3., 4.),
        )));

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    fn hash<T>(obj: T) -> u64
    where
        T: Hash,
    {
        let mut hasher = DefaultHasher::new();
        obj.hash(&mut hasher);
        hasher.finish()
    }
}
