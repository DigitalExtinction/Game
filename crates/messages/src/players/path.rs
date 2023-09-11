use std::mem::size_of;

use bincode::{Decode, Encode};
use de_types::path::Path;
use glam::Vec2;
use thiserror::Error;

use super::Vec2Net;

const MAX_PATH_SIZE: usize = 480;

#[derive(Debug, Encode, Decode)]
pub struct PathNet(Vec<Vec2Net>);

impl TryFrom<&Path> for PathNet {
    type Error = PathError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let waypoints = path.waypoints();

        if waypoints.is_empty() {
            return Err(PathError::Empty);
        }

        let size = waypoints.len() * size_of::<Vec2Net>();
        if size > MAX_PATH_SIZE {
            return Err(PathError::TooLarge {
                size,
                max_size: MAX_PATH_SIZE,
            });
        }

        Ok(Self(waypoints.iter().map(|&p| p.into()).collect()))
    }
}

impl From<&PathNet> for Path {
    fn from(path: &PathNet) -> Self {
        let mut waypoints: Vec<Vec2> = Vec::with_capacity(path.0.len());
        let mut length = 0.;

        for &point in &path.0 {
            let point = point.into();
            if let Some(prev) = waypoints.last() {
                length += prev.distance(point);
            }
            waypoints.push(point);
        }

        Path::new(length, waypoints)
    }
}

#[derive(Debug, Error)]
pub enum PathError {
    #[error("The path is empty")]
    Empty,
    #[error("Too many path way-points: {size} bytes > {max_size} bytes")]
    TooLarge { size: usize, max_size: usize },
}
