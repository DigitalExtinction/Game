use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use glam::Vec2;

pub enum Points {
    Hundred,
    Thousand,
    TenThousand,
    HundredThousand,
}

impl TryFrom<u32> for Points {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            100 => Ok(Self::Hundred),
            1000 => Ok(Self::Thousand),
            10_000 => Ok(Self::TenThousand),
            100_000 => Ok(Self::HundredThousand),
            _ => Err("Invalid number of points"),
        }
    }
}

impl From<Points> for u32 {
    fn from(value: Points) -> Self {
        match value {
            Points::Hundred => 100,
            Points::Thousand => 1000,
            Points::TenThousand => 10_000,
            Points::HundredThousand => 100_000,
        }
    }
}

pub fn load_points(number: Points, max_value: f32) -> Vec<Vec2> {
    let number: u32 = number.into();

    let mut points_path: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    points_path.push("test_data");
    points_path.push(format!("{number}-points.txt"));
    let reader = BufReader::new(File::open(points_path).unwrap());

    let mut points = Vec::with_capacity(number as usize);
    for line in reader.lines() {
        let line = line.unwrap();
        let mut numbers = line.split_whitespace();
        let x: f32 = numbers.next().unwrap().parse().unwrap();
        let y: f32 = numbers.next().unwrap().parse().unwrap();
        points.push(max_value * Vec2::new(x, y));
    }
    points
}
