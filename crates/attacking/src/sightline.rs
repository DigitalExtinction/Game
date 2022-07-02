use bevy::{ecs::system::SystemParam, prelude::Entity};
use de_index::SpatialQuery;
use de_terrain::TerrainCollider;
use parry3d::query::Ray;

#[derive(SystemParam)]
pub(crate) struct LineOfSight<'w, 's> {
    terrain: TerrainCollider<'w, 's>,
    entities: SpatialQuery<'w, 's, Entity>,
}

impl<'w, 's> LineOfSight<'w, 's> {
    pub(crate) fn hit(&self, ray: &Ray, max_toi: f32) -> Hit {
        let entity_hit = match self.entities.cast_ray(ray, max_toi) {
            Some(intersection) => Hit::new(intersection.toi(), Some(intersection.entity())),
            None => Hit::new(max_toi, None),
        };
        let terrain_hit = match self.terrain.cast_ray(ray, max_toi) {
            Some(intersection) => Hit::new(intersection.toi, None),
            None => Hit::new(max_toi, None),
        };

        if entity_hit.toi() < terrain_hit.toi() {
            entity_hit
        } else {
            terrain_hit
        }
    }
}

pub(crate) struct Hit {
    toi: f32,
    entity: Option<Entity>,
}

impl Hit {
    fn new(toi: f32, entity: Option<Entity>) -> Self {
        Self { entity, toi }
    }

    pub(crate) fn toi(&self) -> f32 {
        self.toi
    }

    pub(crate) fn entity(&self) -> Option<Entity> {
        self.entity
    }
}
