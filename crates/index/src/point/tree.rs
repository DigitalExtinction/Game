use ahash::AHashMap;
use bevy::ecs::entity::Entity;
use glam::Vec2;
use kiddo::{distance::squared_euclidean, float::kdtree::KdTree};

pub(super) struct Tree {
    entity_to_index: AHashMap<Entity, u16>,
    slots: Vec<Slot>,
    tree: KdTree<f32, u16, 2, 16, u16>,
}

impl Tree {
    pub(super) fn new() -> Self {
        Self {
            entity_to_index: AHashMap::new(),
            slots: Vec::new(),
            tree: KdTree::new(),
        }
    }

    // TODO docs + panics
    pub(super) fn add(&mut self, entity: Entity, point: Vec2) {
        debug_assert!(self.slots.len() <= u16::MAX as usize);

        let index = self.slots.len() as u16;
        self.entity_to_index.insert(entity, index);

        let coords = point.to_array();
        let slot = Slot { entity, coords };
        self.slots.push(slot);

        self.tree.add(&coords, index);
    }

    // TODO docs + panics
    // TODO tests
    pub(super) fn remove(&mut self, entity: Entity) {
        let index = self.entity_to_index.remove(&entity).unwrap();
        let slot = self.slots.swap_remove(index as usize);
        self.tree.remove(&slot.coords, index);

        if !self.slots.is_empty() {
            // FIXME: https://github.com/sdd/kiddo/issues/93
            let updated_index = self.slots.len() as u16;
            let updated_slot = &self.slots[index as usize];
            self.tree.remove(&updated_slot.coords, updated_index);
            self.tree.add(&updated_slot.coords, index);
        }
    }

    // TODO document + panics
    pub(super) fn update(&mut self, entity: Entity, point: Vec2) {
        // TODO keep precise coordinates inside slots but update the tree only
        // if the change is large enough

        let index = *self.entity_to_index.get(&entity).unwrap();
        let slot = &mut self.slots[index as usize];

        self.tree.remove(&slot.coords, index);
        slot.coords = point.to_array();
        self.tree.add(&slot.coords, index);
    }

    // TODO docs
    pub(super) fn within_unsorted(&self, point: Vec2, dist: f32) -> Vec<Entity> {
        // TODO inflate dist by precision inside tree + further filter based on
        // precise coords

        let slots = &self.slots;
        self.tree
            .within_unsorted(&point.to_array(), dist.powi(2), &squared_euclidean)
            .iter()
            .map(|candidate| slots[candidate.item as usize].entity)
            .collect()
    }
}

struct Slot {
    entity: Entity,
    coords: [f32; 2],
}
