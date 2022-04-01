use anyhow::bail;
use bevy::{prelude::Transform, reflect::TypeUuid};
use glam::{Quat, Vec3};
use serde::Deserialize;

#[derive(TypeUuid, Deserialize)]
#[uuid = "2f2f3f01-8184-4824-beab-50ed0d81550e"]
pub struct MapDescription {
    pub size: MapSize,
    objects: Vec<MapObjectDescription>,
}

impl MapDescription {
    pub fn validate(&self) -> anyhow::Result<()> {
        if !self.size.0.is_finite() {
            bail!("Map size has to be finite, got: {}", self.size.0);
        }
        if self.size.0 <= 0. {
            bail!("Map size has to be positive, got: {}", self.size.0);
        }

        for object in &self.objects {
            let x = object.position[0];
            let y = object.position[1];
            if x < 0. || x > self.size.0 || y < 0. || y > self.size.0 {
                bail!("An object is placed outside of the map: ({}, {})", x, y);
            }
        }

        Ok(())
    }
}

impl MapDescription {
    pub fn objects(&self) -> &[MapObjectDescription] {
        self.objects.as_slice()
    }
}

#[derive(Clone, Copy, Debug, TypeUuid, Deserialize)]
#[uuid = "bbf80d94-c4de-4c7c-9bdc-552ef25aff4e"]
pub struct MapSize(pub f32);

#[derive(Deserialize)]
pub struct MapObjectDescription {
    object_type: MapObjectType,
    position: [f32; 2],
    rotation: f32,
}

impl MapObjectDescription {
    pub fn object_type(&self) -> MapObjectType {
        self.object_type
    }

    pub fn transform(&self) -> Transform {
        let translation = Vec3::new(self.position[0], 0., self.position[1]);
        let rotation = Quat::from_rotation_y(self.rotation);
        Transform {
            translation,
            rotation,
            ..Default::default()
        }
    }
}

#[derive(Copy, Clone, Deserialize)]
pub enum MapObjectType {
    Tree,
}
