use bevy::prelude::Component;
use glam::Vec2;

#[derive(Clone, Copy, Component)]
pub struct PathTarget {
    location: Vec2,
    properties: PathQueryProps,
    permanent: bool,
}

impl PathTarget {
    /// Crate new desired path target.
    ///
    /// # Arguments
    ///
    /// * `location` - desired target location of the path.
    ///
    /// * `properties` - configuration of the path search.
    ///
    /// * `permanent` - whether the entity should try to reach the target
    ///   indefinitely.
    pub fn new(location: Vec2, properties: PathQueryProps, permanent: bool) -> Self {
        Self {
            location,
            properties,
            permanent,
        }
    }

    pub fn location(&self) -> Vec2 {
        self.location
    }

    pub fn properties(&self) -> PathQueryProps {
        self.properties
    }

    pub fn permanent(&self) -> bool {
        self.permanent
    }
}

#[derive(Clone, Copy)]
pub struct PathQueryProps {
    distance: f32,
    max_distance: f32,
}

impl PathQueryProps {
    /// Create new query properties. When possible, a path whose end is between
    /// `distance` and `max_distance` from the target will be searched for.
    /// Paths whose end is closer to the target are preferred.
    ///
    /// # Arguments
    ///
    /// * `distance` - desired path finishes this distance from the target. If
    ///   the target is already closer than `distance` an empty path is desired
    ///   (rather than a path going away from the target).
    ///
    /// * `max_distance` - when all points up to `max_distance` from the target
    ///   are unreachable, it is desired that no path is found. This value must
    ///   be equal or greater than `distance`.
    ///
    /// # Panics
    ///
    /// * May panic if `distance` is not a finite non-negative number.
    ///
    /// * May panic if `max_distance` is not a positive infinity or a finite
    ///   non-negative number.
    ///
    /// * May panic if `distance` is greater than `max_distance`.
    pub fn new(distance: f32, max_distance: f32) -> Self {
        debug_assert!(distance.is_finite());
        debug_assert!(distance >= 0.);
        debug_assert!(max_distance >= 0.);
        debug_assert!(max_distance >= distance);
        Self {
            distance,
            max_distance,
        }
    }

    /// Crate new query properties with both distance and max distance equal to
    /// 0. See [`Self::new`].
    pub fn exact() -> Self {
        Self::new(0., 0.)
    }

    pub fn distance(&self) -> f32 {
        self.distance
    }

    pub fn max_distance(&self) -> f32 {
        self.max_distance
    }
}
