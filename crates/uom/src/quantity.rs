use std::{
    cmp::Ordering,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use thiserror::Error;

pub type Unit = i32;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum QuantityValueError {
    #[error("got NaN")]
    NaN,
}

/// A quantity with associated units.
///
/// The units are either base SI units and several extensions (for example m,
/// s, px) or derived units (for example rad, m/s⁻²). Only unit powers up to
/// +/-7 are supported: id est m² or m⁻² are supported but m⁸ is not.
#[derive(Debug, Clone, Copy)]
pub struct Quantity<const U: Unit>(pub(crate) f32);

impl<const U: Unit> Quantity<U> {
    pub const ZERO: Self = Quantity(0.);
    pub const ONE: Self = Quantity(1.);

    /// Creates a new quantity without checking the value.
    ///
    /// It is expected that the value is not a NaN. If NaN is given, the type
    /// might behave strangely or panic during some of the operations.
    pub const fn new_unchecked(value: f32) -> Self {
        Self(value)
    }

    /// Crates a new quantity.
    ///
    /// # Panics
    ///
    /// Panics if `value` is NaN.
    pub fn new(value: f32) -> Self {
        panic_on_invalid(value);
        Self(value)
    }

    /// Returns a new quantity with absolute value of `self`.
    pub fn abs(&self) -> Self {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        Self::new(self.0.abs())
    }

    pub const fn inner(&self) -> f32 {
        self.0
    }
}

impl<const U: Unit> PartialEq for Quantity<U> {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        #[cfg(debug_assertions)]
        panic_on_invalid(other.0);
        self.0 == other.0
    }
}

impl<const U: Unit> From<Quantity<U>> for f32 {
    fn from(quantity: Quantity<U>) -> f32 {
        #[cfg(debug_assertions)]
        panic_on_invalid(quantity.0);
        quantity.0
    }
}

impl<const U: Unit> TryFrom<f32> for Quantity<U> {
    type Error = QuantityValueError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.is_nan() {
            Err(QuantityValueError::NaN)
        } else {
            Ok(Self(value))
        }
    }
}

impl<const U: Unit> Eq for Quantity<U> {}

impl<const U: Unit> PartialOrd for Quantity<U> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const U: Unit> Ord for Quantity<U> {
    fn cmp(&self, other: &Self) -> Ordering {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        #[cfg(debug_assertions)]
        panic_on_invalid(other.0);
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl<const U: Unit> Neg for Quantity<U> {
    type Output = Self;

    fn neg(self) -> Self {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        Self(-self.0)
    }
}

impl<const U: Unit> Add for Quantity<U> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        #[cfg(debug_assertions)]
        panic_on_invalid(other.0);
        Self(self.0 + other.0)
    }
}

impl<const U: Unit> Sub for Quantity<U> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        #[cfg(debug_assertions)]
        panic_on_invalid(other.0);
        Self(self.0 - other.0)
    }
}

impl<const U: Unit> AddAssign for Quantity<U> {
    fn add_assign(&mut self, other: Self) {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        #[cfg(debug_assertions)]
        panic_on_invalid(other.0);
        self.0 += other.0;
    }
}

impl<const U: Unit> SubAssign for Quantity<U> {
    fn sub_assign(&mut self, other: Self) {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        #[cfg(debug_assertions)]
        panic_on_invalid(other.0);
        self.0 -= other.0;
    }
}

impl<const U: Unit> Mul<f32> for Quantity<U> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self::new(self.0 * rhs)
    }
}

impl<const U: Unit> Mul<Quantity<U>> for f32 {
    type Output = Quantity<U>;

    fn mul(self, rhs: Quantity<U>) -> Quantity<U> {
        Quantity::<U>::new(self * rhs.0)
    }
}

impl<const U: Unit> MulAssign<f32> for Quantity<U> {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
        panic_on_invalid(self.0);
    }
}

impl<const U: Unit> Div<f32> for Quantity<U> {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self::new(self.0 / rhs)
    }
}

impl<const U: Unit> DivAssign<f32> for Quantity<U> {
    fn div_assign(&mut self, rhs: f32) {
        self.0 /= rhs;
        panic_on_invalid(self.0);
    }
}

#[inline]
fn panic_on_invalid(value: f32) {
    if value.is_nan() {
        panic!("Quantity cannot hold a NaN value");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_unchecked() {
        let a: Quantity<1> = Quantity::new_unchecked(2.5);
        let b: Quantity<1> = Quantity::new_unchecked(-4.0);
        assert_eq!(f32::from(a), 2.5);
        assert_eq!(f32::from(b), -4.0);
    }

    #[test]
    fn test_new() {
        let q: Quantity<2> = Quantity::new(5.0);
        assert_eq!(f32::from(q), 5.0);
    }

    #[test]
    #[should_panic]
    fn test_new_panic() {
        Quantity::<3>::new(f32::NAN);
    }

    #[test]
    fn test_eq() {
        let a: Quantity<2> = Quantity::new(5.0);
        let b: Quantity<2> = Quantity::new(5.0);
        let c: Quantity<2> = Quantity::new(-5.0);
        assert_eq!(a, b);
        assert_ne!(b, c);
    }

    #[test]
    fn test_try_from() {
        let a: Quantity<17> = Quantity::try_from(5.5).unwrap();
        assert_eq!(f32::from(a), 5.5);
        let b: Result<Quantity<17>, QuantityValueError> = Quantity::try_from(f32::NAN);
        assert_eq!(b.err().unwrap(), QuantityValueError::NaN);
    }

    #[test]
    fn test_ord() {
        let a: Quantity<42> = Quantity::new(5.5);
        let b: Quantity<42> = Quantity::new(5.5);
        let c: Quantity<42> = Quantity::new(-5.5);
        assert_eq!(a.partial_cmp(&b).unwrap(), a.cmp(&b));
        assert_eq!(a.partial_cmp(&c).unwrap(), a.cmp(&c));
        assert_eq!(c.partial_cmp(&a).unwrap(), c.cmp(&a));
        assert_eq!(a.cmp(&b), Ordering::Equal);
        assert_eq!(a.cmp(&c), Ordering::Greater);
        assert_eq!(c.cmp(&a), Ordering::Less);
    }

    #[test]
    fn test_neg() {
        let a: Quantity<42> = -Quantity::new(69.42);
        assert_eq!(f32::from(a), -69.42);
    }

    #[test]
    fn test_add() {
        let a: Quantity<42> = Quantity::new(1.);
        let b: Quantity<42> = Quantity::new(-1.);
        assert_eq!(a + b, Quantity::<42>::ZERO);
        assert_eq!(f32::from(a + b), 0.);
    }

    #[test]
    fn test_sub() {
        let a: Quantity<42> = Quantity::new(1.);
        let b: Quantity<42> = Quantity::new(-1.);
        assert_eq!(f32::from(a - b), 2.);
    }

    #[test]
    fn test_add_assign() {
        let mut a: Quantity<42> = Quantity::new(1.);
        a += Quantity::new(3.);
        assert_eq!(f32::from(a), 4.);
    }

    #[test]
    fn test_sub_assign() {
        let mut a: Quantity<42> = Quantity::new(1.);
        a -= Quantity::new(3.);
        assert_eq!(f32::from(a), -2.);
    }

    #[test]
    fn test_mul_f32() {
        let a: Quantity<42> = Quantity::new(2.);
        assert_eq!(7.1 * a, a * 7.1);
        assert_eq!(f32::from(7.1 * a), 14.2);
    }

    #[test]
    fn test_mul_assign_f32() {
        let mut a: Quantity<42> = Quantity::new(3.);
        a *= 7.;
        assert_eq!(f32::from(a), 21.);
    }

    #[test]
    fn test_div_f32() {
        let a: Quantity<42> = Quantity::new(3.);
        assert_eq!(f32::from(a / 2.), 1.5);
    }

    #[test]
    fn test_div_assign_f32() {
        let mut a: Quantity<42> = Quantity::new(28.);
        a /= 7.;
        assert_eq!(f32::from(a), 4.);
    }
}
