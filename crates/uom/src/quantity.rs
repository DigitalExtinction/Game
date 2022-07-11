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

pub trait QuantityValue: Copy {
    fn validate(self) -> Result<(), QuantityValueError>;
}

/// A quantity with associated units.
///
/// The units are either base SI units and several extensions (for example m,
/// s, px) or derived units (for example rad, m/s⁻²). Only unit powers up to
/// +/-7 are supported: id est m² or m⁻² are supported but m⁸ is not.
#[derive(Debug)]
pub struct Quantity<V: QuantityValue, const U: Unit>(pub(crate) V);

impl<V: QuantityValue, const U: Unit> Quantity<V, U> {
    /// Crates a new quantity.
    ///
    /// # Panics
    ///
    /// Panics if `value` is NaN.
    pub(crate) fn new(value: V) -> Self {
        panic_on_invalid(value);
        Self(value)
    }
}

impl<V: QuantityValue, const U: Unit> Clone for Quantity<V, U> {
    fn clone(&self) -> Self {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        Self(self.0)
    }
}

impl<V: QuantityValue, const U: Unit> Copy for Quantity<V, U> {}

impl<V: QuantityValue + PartialEq, const U: Unit> PartialEq for Quantity<V, U> {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        #[cfg(debug_assertions)]
        panic_on_invalid(other.0);
        self.0 == other.0
    }
}

impl<V: QuantityValue + PartialOrd, const U: Unit> PartialOrd for Quantity<V, U> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        #[cfg(debug_assertions)]
        panic_on_invalid(other.0);
        self.0.partial_cmp(&other.0)
    }
}

impl<V: QuantityValue + Neg<Output = V>, const U: Unit> Neg for Quantity<V, U> {
    type Output = Self;

    fn neg(self) -> Self {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        Self(-self.0)
    }
}

impl<V: QuantityValue + Add<Output = V>, const U: Unit> Add for Quantity<V, U> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        #[cfg(debug_assertions)]
        panic_on_invalid(other.0);
        Self(self.0 + other.0)
    }
}

impl<V: QuantityValue + Sub<Output = V>, const U: Unit> Sub for Quantity<V, U> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        #[cfg(debug_assertions)]
        panic_on_invalid(other.0);
        Self(self.0 - other.0)
    }
}

impl<V: QuantityValue + AddAssign, const U: Unit> AddAssign for Quantity<V, U> {
    fn add_assign(&mut self, other: Self) {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        #[cfg(debug_assertions)]
        panic_on_invalid(other.0);
        self.0 += other.0;
    }
}

impl<V: QuantityValue + SubAssign, const U: Unit> SubAssign for Quantity<V, U> {
    fn sub_assign(&mut self, other: Self) {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        #[cfg(debug_assertions)]
        panic_on_invalid(other.0);
        self.0 -= other.0;
    }
}

impl<R: QuantityValue, O: QuantityValue, V: QuantityValue + Mul<R, Output = O>, const U: Unit>
    Mul<R> for Quantity<V, U>
{
    type Output = Quantity<O, U>;

    fn mul(self, rhs: R) -> Self::Output {
        Self::Output::new(self.0 * rhs)
    }
}

impl<V: QuantityValue, const U: Unit> Mul<Quantity<V, U>> for f32
where
    f32: QuantityValue + Mul<V, Output = V>,
{
    type Output = Quantity<V, U>;

    fn mul(self, rhs: Quantity<V, U>) -> Quantity<V, U> {
        Quantity::<V, U>::new(self * rhs.0)
    }
}

impl<V: QuantityValue + MulAssign<f32>, const U: Unit> MulAssign<f32> for Quantity<V, U> {
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
        panic_on_invalid(self.0);
    }
}

impl<V: QuantityValue + Div<f32, Output = V>, const U: Unit> Div<f32> for Quantity<V, U> {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self::new(self.0 / rhs)
    }
}

impl<V: QuantityValue + DivAssign<f32>, const U: Unit> DivAssign<f32> for Quantity<V, U> {
    fn div_assign(&mut self, rhs: f32) {
        self.0 /= rhs;
        panic_on_invalid(self.0);
    }
}

#[inline]
pub(crate) fn panic_on_invalid(value: impl QuantityValue) {
    if let Err(error) = value.validate() {
        panic!("{:?}", error);
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
