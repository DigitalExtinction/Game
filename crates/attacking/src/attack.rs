use std::{cmp::Ordering, collections::BinaryHeap};

use bevy::prelude::*;
use de_behaviour::AttackTarget;
use de_core::state::GameState;
use de_objects::LaserCannon;
use iyes_loopless::prelude::*;
use parry3d::query::Ray;

use crate::{laser::FireEvent, sightline::LineOfSight, AttackingLabels};

pub(crate) struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::new()
                .with_system(
                    update
                        .run_in_state(GameState::Playing)
                        .label(AttackingLabels::Update),
                )
                .with_system(
                    aim_and_fire
                        .run_in_state(GameState::Playing)
                        .label(AttackingLabels::Aim)
                        .before(AttackingLabels::Fire),
                ),
        );
    }
}

fn update(time: Res<Time>, mut cannons: Query<&mut LaserCannon>) {
    for mut cannon in cannons.iter_mut() {
        cannon.timer_mut().tick(time.delta());
    }
}

fn aim_and_fire(
    mut attackers: Query<(Entity, &GlobalTransform, &mut LaserCannon, &AttackTarget)>,
    targets: Query<&GlobalTransform>,
    sightline: LineOfSight,
    mut events: EventWriter<FireEvent>,
) {
    let attackers = attackers.iter_mut();
    let mut fire_queue = BinaryHeap::new();

    for (attacker, attacker_transform, mut cannon, target) in attackers {
        let target_transform = match targets.get(target.entity()) {
            Ok(transform) => transform,
            Err(_) => continue,
        };

        let muzzle = attacker_transform.translation + cannon.muzzle();
        // TODO do not aim at the object position but at center of its body
        let target_position = target_transform.translation;
        let to_target = (target_position - muzzle)
            .try_normalize()
            .expect("Attacker and target to close together");
        let ray = Ray::new(muzzle.into(), to_target.into());

        let hit = sightline.hit(&ray, cannon.range());
        if hit.entity().map_or(true, |e| e != target.entity()) {
            cannon.timer_mut().reset();
        }

        if cannon.timer_mut().check_and_update() {
            fire_queue.push(FireItem::new(attacker, ray, cannon.into_inner()));
        }
    }

    while let Some(mut fire_item) = fire_queue.pop() {
        if fire_item.fire(&mut events) {
            fire_queue.push(fire_item);
        }
    }
}

struct FireItem<'a> {
    attacker: Entity,
    ray: Ray,
    cannon: &'a mut LaserCannon,
}

impl<'a> FireItem<'a> {
    fn new(attacker: Entity, ray: Ray, cannon: &'a mut LaserCannon) -> Self {
        Self {
            attacker,
            ray,
            cannon,
        }
    }

    fn fire(&mut self, events: &mut EventWriter<FireEvent>) -> bool {
        events.send(FireEvent::new(
            self.attacker,
            self.ray,
            self.cannon.range(),
            self.cannon.damage(),
        ));
        self.cannon.timer_mut().check_and_update()
    }
}

impl<'a> Ord for FireItem<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        let ordering = self.cannon.timer().cmp(other.cannon.timer());
        if let Ordering::Equal = ordering {
            // Make it more deterministic, objects with smaller coordinates
            // have disadvantage.
            self.ray
                .origin
                .partial_cmp(&other.ray.origin)
                .unwrap_or(Ordering::Equal)
        } else {
            ordering
        }
    }
}

impl<'a> PartialOrd for FireItem<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> PartialEq for FireItem<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.ray.origin == other.ray.origin && self.cannon.timer() == other.cannon.timer()
    }
}

impl<'a> Eq for FireItem<'a> {}
