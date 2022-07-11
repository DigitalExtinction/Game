use std::ops::Mul;

use glam::{Vec2, Vec3};

use crate::quantity::{panic_on_invalid, Quantity, QuantityValue, QuantityValueError, Unit};

macro_rules! impl_vec {
    ($type:ty, $($coord:ident),+) => {
        impl QuantityValue for $type {
            fn validate(self) -> Result<(), QuantityValueError> {
                if self.is_nan() {
                    Err(QuantityValueError::NaN)
                } else {
                    Ok(())
                }
            }
        }

        impl<const U: Unit> Quantity<$type, U> {
            pub const ZERO: Self = Quantity(<$type>::ZERO);
            pub const ONE: Self = Quantity(<$type>::ONE);

            pub const fn new_unchecked($($coord: f32),+) -> Self {
                Self(<$type>::new($($coord),+))
            }

            $(
                pub fn $coord(&self) -> Quantity<f32, U> {
                    #[cfg(debug_assertions)]
                    panic_on_invalid(self.0.$coord);
                    Quantity(self.0.$coord)
                }
            )+

            pub fn min(&self, rhs: Self) -> Self {
                Self(self.0.min(rhs.0))
            }

            pub fn max(&self, rhs: Self) -> Self {
                Self(self.0.max(rhs.0))
            }

            pub fn clamp(&self, min: Self, max: Self) -> Self {
                Self(self.0.clamp(min.0, max.0))
            }
        }

        impl<const U: Unit> From<Quantity<$type, U>> for $type {
            fn from(value: Quantity<$type, U>) -> $type {
                value.0
            }
        }

        impl<const U: Unit> TryFrom<$type> for Quantity<$type, U> {
            type Error = QuantityValueError;

            fn try_from(value: $type) -> Result<Self, Self::Error> {
                if value.is_nan() {
                    Err(QuantityValueError::NaN)
                } else {
                    Ok(Self(value))
                }
            }
        }

        impl<V: QuantityValue, O: QuantityValue, const U: Unit> Mul<Quantity<V, U>> for $type
        where
            $type: QuantityValue + Mul<V, Output = O>,
        {
            type Output = Quantity<O, U>;

            fn mul(self, rhs: Quantity<V, U>) -> Self::Output {
                Self::Output::new(self * rhs.0)
            }
        }
    };
}

impl_vec!(Vec2, x, y);
impl_vec!(Vec3, x, y, z);
