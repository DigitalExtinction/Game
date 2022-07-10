use std::f32::consts::{FRAC_PI_2, TAU};

use crate::units::Radian;

impl Radian {
    /// The angle equal to π/2.
    pub const FRAC_PI_2: Self = Self(FRAC_PI_2);

    /// Returns a new angle normalized to a values between 0.0 (inclusive) and
    /// the full circle constant τ (exclusive).
    pub fn normalized(&self) -> Self {
        debug_assert!(self.0.is_finite());
        Self::new(self.0.rem_euclid(TAU))
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn test_normalized() {
        let angle = Radian::try_from(5. * PI).unwrap();
        assert_relative_eq!(f32::from(angle), 5. * PI);
        assert_relative_eq!(f32::from(angle.normalized()), PI);
    }
}
