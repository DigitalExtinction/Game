use std::marker::PhantomData;

use bevy::prelude::{Component, Entity};

/// Base for exponential (per second) rate of entities Vec allocated memory
/// decay.
const DECAY_RATE: f32 = 0.8;

#[derive(Component)]
pub(crate) struct DecayingCache<M> {
    entities: Vec<Entity>,
    capacity: f32,
    _m: PhantomData<M>,
}

impl<M> DecayingCache<M> {
    pub(crate) fn entities(&self) -> &[Entity] {
        self.entities.as_slice()
    }

    pub(crate) fn clear(&mut self) {
        self.entities.clear()
    }

    pub(crate) fn extend<I>(&mut self, entities: I)
    where
        I: IntoIterator<Item = Entity>,
    {
        self.entities.extend(entities);
    }

    /// This functions exponentially decreases desired capacity of the
    /// underlying vector. The minimum desired capacity is lower bounded by the
    /// underlying vector's length.
    ///
    /// Actual capacity is reduced only if desired capacity is at least 2x
    /// smaller than current capacity.
    ///
    /// # Arguments
    ///
    /// * `delta_seconds` - time passed since the last call to this method.
    pub(crate) fn decay(&mut self, delta_seconds: f32) {
        debug_assert!(delta_seconds >= 0.);
        let factor = DECAY_RATE.powf(delta_seconds);
        self.capacity = (self.capacity * factor).max(self.entities.len() as f32);

        let desired = self.capacity.ceil() as usize;
        debug_assert!(desired >= self.entities.len());
        if desired <= (self.entities.capacity() / 2) {
            self.entities.shrink_to(desired);
        }
    }
}

impl<M> Default for DecayingCache<M> {
    fn default() -> Self {
        DecayingCache {
            entities: Vec::new(),
            capacity: 0.,
            _m: PhantomData::default(),
        }
    }
}
