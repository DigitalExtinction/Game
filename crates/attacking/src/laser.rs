use bevy::prelude::*;
use de_core::state::GameState;
use de_objects::Health;
use de_spawner::SpawnerLabels;
use iyes_loopless::prelude::*;
use parry3d::query::Ray;

use crate::{beam::SpawnBeamEvent, sightline::LineOfSight, AttackingLabels};

pub(crate) struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FireEvent>().add_system_to_stage(
            CoreStage::Update,
            fire.run_in_state(GameState::Playing)
                .label(AttackingLabels::Fire)
                .before(SpawnerLabels::Destroyer)
                .before(AttackingLabels::Animate),
        );
    }
}

// TODO docs
// docs ordering & health checking
pub(crate) struct FireEvent {
    attacker: Entity,
    ray: Ray,
    max_toi: f32,
    damage: f32,
}

impl FireEvent {
    pub fn new(attacker: Entity, ray: Ray, max_toi: f32, damage: f32) -> Self {
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
    mut fires: EventReader<FireEvent>,
    mut beams: EventWriter<SpawnBeamEvent>,
    sightline: LineOfSight,
    mut susceptible: Query<&mut Health>,
) {
    for fire in fires.iter() {
        if susceptible
            .get(fire.attacker())
            .map_or(true, |health| health.destroyed())
        {
            continue;
        }

        let hit = sightline.hit(fire.ray(), fire.max_toi());
        beams.send(SpawnBeamEvent::new(Ray::new(
            fire.ray().origin,
            hit.toi() * fire.ray().dir,
        )));
        if let Some(entity) = hit.entity() {
            susceptible.get_mut(entity).unwrap().hit(fire.damage());
        }
    }
}
