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
    /// Looks into a direction up until some furthest point.
    ///
    /// # Arguments
    ///
    /// * `ray` - gives the direction to look.
    ///
    /// * `max_toi` - limits the maximum observable distance. The furthest
    ///   observable point is given by `ray.origin * max_toi`.
    ///
    /// * `observer` - the entity making the observation. This is needed so the
    ///   entity doesn't observe itself.
    pub(crate) fn sight(&self, ray: &Ray, max_toi: f32, observer: Entity) -> Observation {
        // It is more efficient to calculate the terrain hit. Do it first so
        // max_toi can be lowered in case of a hit.
        let hit = match self.terrain.cast_ray(ray, max_toi) {
            Some(intersection) => Observation::new(intersection.toi, None),
            None => Observation::new(max_toi, None),
        };
        self.entities
            .cast_ray(ray, hit.toi(), Some(observer))
            .map(|i| Observation::new(i.toi(), Some(i.entity())))
            .unwrap_or(hit)
    }
}

pub(crate) struct Observation {
    toi: f32,
    entity: Option<Entity>,
}

impl Observation {
    fn new(toi: f32, entity: Option<Entity>) -> Self {
        Self { entity, toi }
    }

    /// Returns visibility along the ray. `toi * direction` is the point of
    /// first obstacle in that direction or the furthest point within range
    /// limited by a max toi parameter.
    pub(crate) fn toi(&self) -> f32 {
        self.toi
    }

    /// The entity observed, if any, along a given ray.
    pub(crate) fn entity(&self) -> Option<Entity> {
        self.entity
    }
}
