use std::ops::Mul;

use crate::quantity::{Quantity, QuantityValue, Unit};

const DIMENSIONLESS: Unit = 0;
const SECOND: Unit = 1;
const METRE: Unit = 1 << 3;
const KILOGRAM: Unit = 1 << 6;
const AMPERE: Unit = 1 << 9;
const KELVIN: Unit = 1 << 12;
const MOLE: Unit = 1 << 15;
const CANDELA: Unit = 1 << 18;
const PIXEL: Unit = 1 << 21;

// Base Units
pub type Second<V> = Quantity<V, SECOND>;
pub type Metre<V> = Quantity<V, METRE>;
pub type Kilogram<V> = Quantity<V, KILOGRAM>;
pub type Ampere<V> = Quantity<V, AMPERE>;
pub type Kelvin<V> = Quantity<V, KELVIN>;
pub type Mole<V> = Quantity<V, MOLE>;
pub type Candela<V> = Quantity<V, CANDELA>;
pub type Pixel<V> = Quantity<V, PIXEL>;

// Derived units
pub type InverseSecond<V> = Quantity<V, { -SECOND }>;
pub type LogicalPixel<V> = Quantity<V, PIXEL>;
pub type InverseLogicalPixel<V> = Quantity<V, { -PIXEL }>;
pub type Radian<V> = Quantity<V, DIMENSIONLESS>;

macro_rules! impl_mul_inverse {
    ($units:expr) => {
        impl<V1, V2, O> Mul<Quantity<V2, { -$units }>> for Quantity<V1, $units>
        where
            V1: QuantityValue + Mul<V2, Output = O>,
            V2: QuantityValue,
            O: QuantityValue,
        {
            type Output = O;

            fn mul(self, rhs: Quantity<V2, { -$units }>) -> Self::Output {
                self.0 * rhs.0
            }
        }

        impl<V1, V2, O> Mul<Quantity<V2, $units>> for Quantity<V1, { -$units }>
        where
            V1: QuantityValue + Mul<V2, Output = O>,
            V2: QuantityValue,
            O: QuantityValue,
        {
            type Output = O;

            fn mul(self, rhs: Quantity<V2, $units>) -> Self::Output {
                self.0 * rhs.0
            }
        }
    };
}

// Due to combinatorial explosion, only needed multiplications are implemented.
impl_mul_inverse!(SECOND);
impl_mul_inverse!(PIXEL);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inverse_second() {
        let a = Second::try_from(20.).unwrap();
        let b = InverseSecond::try_from(20.).unwrap();
        assert_eq!(a * b, 400.);
    }
}
