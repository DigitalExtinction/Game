use glam::Vec2;

#[derive(Clone, Copy)]
pub struct PathTarget {
    location: Vec2,
    distance: f32,
}

impl PathTarget {
    /// Crate new desired path target.
    ///
    /// # Arguments
    ///
    /// * `location` - desired target location of the path.
    ///
    /// * `distance` - desired path should finish this distance from
    ///   `location`. If `location` is closer than `distance` an empty path is
    ///   desired (rather then going further from `location`).
    ///
    /// # Panics
    ///
    /// May panic if distance is not a finite non-negative number.
    pub fn new(location: Vec2, distance: f32) -> Self {
        debug_assert!(distance.is_finite());
        debug_assert!(distance >= 0.);
        Self { location, distance }
    }

    /// Create a new desired path target with 0 distance to a location. See
    /// [`Self::new`].
    pub fn exact(location: Vec2) -> Self {
        Self::new(location, 0.)
    }

    pub fn location(&self) -> Vec2 {
        self.location
    }

    pub fn distance(&self) -> f32 {
        self.distance
    }
}
