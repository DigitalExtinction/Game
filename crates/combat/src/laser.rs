use bevy::prelude::*;
use de_core::{baseset::GameSet, gamestate::GameState};
use de_objects::Health;
use de_signs::UpdateBarValueEvent;
use de_spawner::SpawnerSet;
use parry3d::query::Ray;

use crate::{sightline::LineOfSight, trail::TrailEvent, AttackingSet};

pub(crate) struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LaserFireEvent>().add_system(
            fire.in_base_set(GameSet::Update)
                .run_if(in_state(GameState::Playing))
                .in_set(AttackingSet::Fire)
                .before(SpawnerSet::Destroyer),
        );
    }
}

/// Send this even to fire a laser from an entity in a direction.
///
/// This event is ignored when the attacker has 0 health or no longer exists.
/// Thus ordering of the events is important.
pub(crate) struct LaserFireEvent {
    attacker: Entity,
    ray: Ray,
    max_toi: f32,
    damage: f32,
}

impl LaserFireEvent {
    /// Crates a new laser fire event.
    ///
    /// # Arguments
    ///
    /// * `attacker` - the firing entity.
    ///
    /// * `ray` - laser beam origin and direction.
    ///
    /// * `max_toi` - this limits maximum distance to hit unit. The furthest
    ///   point is given by formula `ray.origin + max_toi * ray.dir`.
    ///
    /// * `damage` - if an entity is hit, its health will be lowered by this
    ///   amount.
    #[allow(dead_code)]
    pub(crate) fn new(attacker: Entity, ray: Ray, max_toi: f32, damage: f32) -> Self {
        Self {
            attacker,
            ray,
            max_toi,
            damage,
        }
    }

    fn attacker(&self) -> Entity {
        self.attacker
    }

    fn ray(&self) -> &Ray {
        &self.ray
    }

    fn max_toi(&self) -> f32 {
        self.max_toi
    }

    fn damage(&self) -> f32 {
        self.damage
    }
}

fn fire(
    mut fires: EventReader<LaserFireEvent>,
    sightline: LineOfSight,
    mut susceptible: Query<&mut Health>,
    mut bar: EventWriter<UpdateBarValueEvent>,
    mut trail: EventWriter<TrailEvent>,
) {
    for fire in fires.iter() {
        if susceptible
            .get(fire.attacker())
            .map_or(true, |health| health.destroyed())
        {
            continue;
        }

        let observation = sightline.sight(fire.ray(), fire.max_toi(), fire.attacker());

        trail.send(TrailEvent::new(Ray::new(
            fire.ray().origin,
            observation.toi() * fire.ray().dir,
        )));

        if let Some(entity) = observation.entity() {
            let mut health = susceptible.get_mut(entity).unwrap();
            health.hit(fire.damage());
            bar.send(UpdateBarValueEvent::new(entity, health.fraction()));
        }
    }
}
