use std::ops::Mul;

use crate::quantity::{Quantity, Unit};

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
pub type Second = Quantity<SECOND>;
pub type Metre = Quantity<METRE>;
pub type Kilogram = Quantity<KILOGRAM>;
pub type Ampere = Quantity<AMPERE>;
pub type Kelvin = Quantity<KELVIN>;
pub type Mole = Quantity<MOLE>;
pub type Candela = Quantity<CANDELA>;
pub type Pixel = Quantity<PIXEL>;

// Derived units
pub type InverseSecond = Quantity<{ -SECOND }>;
pub type LogicalPixel = Quantity<PIXEL>;
pub type InverseLogicalPixel = Quantity<{ -PIXEL }>;
pub type Radian = Quantity<DIMENSIONLESS>;

macro_rules! impl_mul_inverse {
    ($units:expr) => {
        impl Mul<Quantity<{ -$units }>> for Quantity<$units> {
            type Output = f32;

            fn mul(self, rhs: Quantity<{ -$units }>) -> Self::Output {
                self.0 * rhs.0
            }
        }

        impl Mul<Quantity<$units>> for Quantity<{ -$units }> {
            type Output = f32;

            fn mul(self, rhs: Quantity<$units>) -> Self::Output {
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
