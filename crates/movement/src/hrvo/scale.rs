//! This module implements conversions from and into fixed-point arithmetic
//! (integer) vectors.
//!
//! To avoid numeric stability issues related to f32 numbers, all vectors and
//! numbers in HRVO implementation ([`super`]) are converted to fractional
//! integer vectors.

use glam::{IVec2, Vec2};

/// Number of fractional (least significant) bits in a number.
const SCALE_BITS: u8 = 10;
const SCALE_I32: i32 = 1 << SCALE_BITS;
const SCALE_MASK: i32 = SCALE_I32 - 1;
const SCALE_F32: f32 = SCALE_I32 as f32;

// Maximum absolute value of any coordinate must be small enough to allow
// non-overflowing dot product between a "unit" vector and a sum of two other
// vectors.
const POINT_MAX: i32 = (i32::MAX - 1) / (4 * SCALE_I32);

/// Returns a fixed-point (i32) vector created from a world space scale (f32)
/// vector.
///
/// # Panics
///
/// May panic if the vector is too large.
pub(super) fn vec_to_scale(vec: Vec2) -> IVec2 {
    let vec = (SCALE_F32 * vec).round().as_ivec2();
    debug_assert!(vec.x.abs() <= POINT_MAX);
    debug_assert!(vec.y.abs() <= POINT_MAX);
    vec
}

/// Returns a fixed-point (i32) scalar created from a world space scale (f32)
/// scalar.
///
/// # Panics
///
/// May panic if the scalar is too large.
pub(super) fn scalar_to_scale(scalar: f32) -> i32 {
    let scalar = (SCALE_F32 * scalar).round() as i32;
    debug_assert!(scalar.abs() <= POINT_MAX);
    scalar
}

/// Returns a world space scale vector (f32) created from a fixed-point (i32)
/// vector.
pub(super) fn vec_from_scale(vec: IVec2) -> Vec2 {
    (vec.as_dvec2() / f64::from(SCALE_F32)).as_vec2()
}

/// Normalizes a vector whose magnitude is scale times larger than it should
/// be.
pub(super) fn vec_div_to_scale(vec: IVec2) -> IVec2 {
    IVec2::new(scalar_div_to_scale(vec.x), scalar_div_to_scale(vec.y))
}

/// Computes fraction of two numbers normalized to fixed-point scale.
pub(super) fn scaled_div_floor(numerator: i32, denominator: i32) -> i32 {
    let numerator: i64 = i64::from(SCALE_I32) * i64::from(numerator);
    let denominator: i64 = denominator.into();

    let d = numerator / denominator;
    let r = numerator % denominator;
    if (r > 0 && denominator < 0) || (r < 0 && denominator > 0) {
        (d - 1) as i32
    } else {
        d as i32
    }
}

pub(super) fn scalar_div_to_scale(value: i32) -> i32 {
    let d = value / SCALE_I32;
    let r = value & SCALE_MASK;
    if value < 0 && r > 0 {
        d - 1
    } else {
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaled_div_floor() {
        assert_eq!(scaled_div_floor(30, 10), 3072);
        assert_eq!(scaled_div_floor(29, 10), 2969);

        assert_eq!(scaled_div_floor(-30, 10), -3072);
        assert_eq!(scaled_div_floor(-29, 10), -2970);

        assert_eq!(scaled_div_floor(29, -10), -2970);
        assert_eq!(scaled_div_floor(30, -10), -3072);

        assert_eq!(scaled_div_floor(-30, -10), 3072);
        assert_eq!(scaled_div_floor(-29, -10), 2969);
    }
}
