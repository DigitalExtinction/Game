//! Provides utilities for (de)serialization and work with grids of floating
//! point values like digital elevation maps.

use std::ops::{Add, Div};

/// Point in a discrete 2D space. The U axis goes from left to right. The V
/// axis goes from top down.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DiscretePoint {
    pub u: u32,
    pub v: u32,
}

impl Add for DiscretePoint {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            u: self.u + other.u,
            v: self.v + other.v,
        }
    }
}

impl Div<u32> for DiscretePoint {
    type Output = Self;

    fn div(self, rhs: u32) -> Self::Output {
        debug_assert!(rhs != 0, "Division by zero!");
        debug_assert!(
            (self.u % rhs) == 0,
            "Coordinate `u = {}` is not divisible by {}.",
            self.u,
            rhs
        );
        debug_assert!(
            (self.v % rhs) == 0,
            "Coordinate `v = {}` is not divisible by {}.",
            self.v,
            rhs
        );
        Self {
            u: self.u / rhs,
            v: self.v / rhs,
        }
    }
}

/// Grid of floating point values.
pub struct ValueGrid {
    values: Vec<f32>,
    size: u16,
}

impl ValueGrid {
    fn check_size(size: u16) {
        if size == 0 {
            panic!("Cannot create grid of size 0.");
        }
        if usize::BITS < 2 * (16 - size.leading_zeros()) {
            panic!("Size is too big.");
        }
    }

    /// Create a new grid filled with 0s.
    ///
    /// # Arguments
    ///
    /// * `size` - Number of columns and rows in the map.
    ///
    /// # Panics
    ///
    /// Panics if `size` is 0
    pub fn with_zeros(size: u16) -> Self {
        // Check first to avoid large memory allocation or an u32 overflow.
        Self::check_size(size);
        let values: Vec<f32> = vec![0.; (size as usize).pow(2)];
        Self::new(values, size)
    }

    fn new(values: Vec<f32>, size: u16) -> Self {
        Self::check_size(size);
        if values.len() != (size as usize).pow(2) {
            panic!("Values Vec has incorrect size.");
        }
        Self { values, size }
    }

    fn index(&self, point: DiscretePoint) -> usize {
        debug_assert!(
            point.u < (self.size as u32),
            "`point.u` is too large: {} >= {}",
            point.u,
            self.size
        );
        debug_assert!(
            point.v < (self.size as u32),
            "`point.v` is too large: {} >= {}",
            point.v,
            self.size
        );
        point.v as usize * self.size as usize + point.u as usize
    }

    pub fn value(&self, point: DiscretePoint) -> f32 {
        self.values[self.index(point)]
    }

    pub fn set_value(&mut self, point: DiscretePoint, value: f32) {
        let index = self.index(point);
        self.values[index] = value;
    }

    pub fn size(&self) -> u16 {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_point_add() {
        let a = DiscretePoint { u: 1, v: 2 };
        let b = DiscretePoint { u: 3, v: 4 };
        assert_eq!(a + b, DiscretePoint { u: 4, v: 6 });
    }

    #[test]
    fn test_pixel_point_div() {
        let point = DiscretePoint { u: 4, v: 16 };
        assert_eq!(point / 2, DiscretePoint { u: 2, v: 8 });
    }

    #[test]
    fn test_value_map() {
        let mut value_map = ValueGrid::with_zeros(5);
        assert_eq!(value_map.value(DiscretePoint { u: 1, v: 2 }), 0.);
        value_map.set_value(DiscretePoint { u: 1, v: 2 }, -1.1);
        assert_eq!(value_map.value(DiscretePoint { u: 1, v: 2 }), -1.1);
        assert_eq!(value_map.value(DiscretePoint { u: 1, v: 3 }), 0.);
    }
}
