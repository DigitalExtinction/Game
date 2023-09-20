use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use glam::Vec2;

/// An enum to allow for safe selection of the number of points to load from the test data.
#[derive(Copy, Clone, Debug)]
pub enum NumPoints {
    OneHundred,
    OneThousand,
    TenThousand,
    OneHundredThousand,
}

impl TryFrom<u32> for NumPoints {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            100 => Ok(Self::OneHundred),
            1000 => Ok(Self::OneThousand),
            10_000 => Ok(Self::TenThousand),
            100_000 => Ok(Self::OneHundredThousand),
            _ => Err("Invalid number of points"),
        }
    }
}

impl From<&NumPoints> for usize {
    fn from(value: &NumPoints) -> Self {
        match value {
            NumPoints::OneHundred => 100,
            NumPoints::OneThousand => 1000,
            NumPoints::TenThousand => 10_000,
            NumPoints::OneHundredThousand => 100_000,
        }
    }
}

impl From<NumPoints> for usize {
    fn from(value: NumPoints) -> Self {
        Self::from(&value)
    }
}

/// Load deterministic points for testing.
///
/// # Arguments
/// * `number` - the selected number of points from the [NumPoints] enum.
/// * `max_value` - the max and min value for the returned point, the numbers returned will be
/// between -max_value and +max_value.
///
/// # Returns
/// A list of Vec2 points with x and y between -max_value and +max_value. This is guaranteed to be
/// deterministic across calls with the same input.
pub fn load_points(number: &NumPoints, max_value: f32) -> Vec<Vec2> {
    let number: usize = number.into();

    let mut points_path: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    points_path.push("test_data");
    points_path.push(format!("{number}-points.txt"));
    let reader = BufReader::new(File::open(points_path).unwrap());

    let mut points = Vec::with_capacity(number);
    for line in reader.lines() {
        let line = line.unwrap();
        let mut numbers = line.split_whitespace();
        let x: f32 = numbers.next().unwrap().parse().unwrap();

        assert!(x.is_finite());
        assert!(x >= 0.);
        assert!(x <= 1.);

        let y: f32 = numbers.next().unwrap().parse().unwrap();

        assert!(y.is_finite());
        assert!(y >= 0.);
        assert!(y <= 1.);

        points.push(max_value * 2. * (Vec2::new(x, y) - 0.5));
    }
    points
}
