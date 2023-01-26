use std::f32::consts::TAU;

use bevy::prelude::Transform;
use de_core::projection::ToAltitude;
use glam::{Quat, Vec2};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{hash::MapHasher, size::MapBounds};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Placement {
    position: Vec2,
    heading: f32,
}

impl Placement {
    pub(crate) fn new(position: Vec2, heading: f32) -> Self {
        Self { position, heading }
    }

    pub(crate) fn update_hash(&self, hasher: &mut MapHasher) {
        hasher.update_vec2(self.position);
        hasher.update_f32(self.heading);
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    /// Produces world to object transform which can be used to position the
    /// object on the map.
    pub fn to_transform(self) -> Transform {
        let rotation = Quat::from_rotation_y(self.heading);
        Transform {
            translation: self.position.to_msl(),
            rotation,
            ..Default::default()
        }
    }

    pub(crate) fn validate(&self, map_bounds: MapBounds) -> Result<(), PlacementValidationError> {
        if !map_bounds.contains(self.position) {
            return Err(PlacementValidationError::OutOfMapBound(self.position));
        }
        if !self.heading.is_finite() || self.heading < 0. || self.heading >= TAU {
            return Err(PlacementValidationError::InvalidHeading(self.heading));
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum PlacementValidationError {
    #[error("position ({}, {}) is out of map bounds", .0.x, .0.y)]
    OutOfMapBound(Vec2),
    #[error("heading must be between 0 and τ (2π), got: {0}")]
    InvalidHeading(f32),
}
