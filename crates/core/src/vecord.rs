use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use glam::Vec2;

/// A wrapper for Vec2Ord that implements [`Ord`], [`Eq`], and [`Hash`] traits.
#[derive(Debug, Copy, Clone)]
pub struct Vec2Ord(pub Vec2);

impl PartialOrd for Vec2Ord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Vec2Ord {
    fn cmp(&self, other: &Self) -> Ordering {
        cmp(self.0.x, other.0.x).then_with(|| cmp(self.0.y, other.0.y))
    }
}

impl PartialEq for Vec2Ord {
    fn eq(&self, other: &Self) -> bool {
        eq(self.0.x, other.0.x) && eq(self.0.y, other.0.y)
    }
}

impl Eq for Vec2Ord {}

impl Hash for Vec2Ord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash(self.0.x, state);
        hash(self.0.y, state);
    }
}

impl From<Vec2> for Vec2Ord {
    fn from(vec: Vec2) -> Self {
        Self(vec)
    }
}

fn eq(a: f32, b: f32) -> bool {
    (a.is_nan() && b.is_nan()) || a == b
}

fn cmp(a: f32, b: f32) -> Ordering {
    a.partial_cmp(&b).unwrap_or_else(|| {
        if a.is_nan() && !b.is_nan() {
            Ordering::Less
        } else if !a.is_nan() && b.is_nan() {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    })
}

fn hash<H: Hasher>(value: f32, state: &mut H) {
    if value.is_nan() {
        // Ensure all NaN representations hash to the same value
        state.write(&f32::to_ne_bytes(f32::NAN));
    } else if value == 0.0 {
        // Ensure both zeroes hash to the same value
        state.write(&f32::to_ne_bytes(0.0f32));
    } else {
        state.write(&f32::to_ne_bytes(value));
    }
}
