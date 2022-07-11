use std::cmp::Ordering;

use crate::quantity::{panic_on_invalid, Quantity, QuantityValue, QuantityValueError, Unit};

impl QuantityValue for f32 {
    fn validate(self) -> Result<(), QuantityValueError> {
        if self.is_nan() {
            Err(QuantityValueError::NaN)
        } else {
            Ok(())
        }
    }
}

impl<const U: Unit> Quantity<f32, U> {
    pub const ZERO: Self = Quantity(0.);
    pub const ONE: Self = Quantity(1.);

    /// Creates a new quantity without checking the value.
    ///
    /// It is expected that the value is not a NaN. If NaN is given, the type
    /// might behave strangely or panic during some of the operations.
    pub const fn new_unchecked(value: f32) -> Self {
        Self(value)
    }

    /// Returns a new quantity with absolute value of `self`.
    pub fn abs(&self) -> Self {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        Self::new(self.0.abs())
    }
}

impl<const U: Unit> From<Quantity<f32, U>> for f32 {
    fn from(quantity: Quantity<f32, U>) -> f32 {
        #[cfg(debug_assertions)]
        panic_on_invalid(quantity.0);
        quantity.0
    }
}

impl<const U: Unit> TryFrom<f32> for Quantity<f32, U> {
    type Error = QuantityValueError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.is_nan() {
            Err(QuantityValueError::NaN)
        } else {
            Ok(Self(value))
        }
    }
}

impl<const U: Unit> Eq for Quantity<f32, U> {}

impl<const U: Unit> Ord for Quantity<f32, U> {
    fn cmp(&self, other: &Self) -> Ordering {
        #[cfg(debug_assertions)]
        panic_on_invalid(self.0);
        #[cfg(debug_assertions)]
        panic_on_invalid(other.0);
        self.0.partial_cmp(&other.0).unwrap()
    }
}
