use bevy::prelude::Component;
use glam::Vec2;

#[derive(Component, Default, Copy, Clone)]
pub(crate) struct Disc {
    center: Vec2,
    radius: f32,
}

impl Disc {
    pub(crate) fn new(center: Vec2, radius: f32) -> Self {
        Self { center, radius }
    }

    pub(crate) fn center(&self) -> Vec2 {
        self.center
    }

    pub(crate) fn radius(&self) -> f32 {
        self.radius
    }

    pub(crate) fn set_center(&mut self, center: Vec2) {
        self.center = center;
    }
}
