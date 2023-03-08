use std::{cmp::Ordering, collections::BinaryHeap};

use bevy::prelude::*;
use de_behaviour::{ChaseSet, ChaseTarget, ChaseTargetComponent, ChaseTargetEvent};
use de_core::{baseset::GameSet, gamestate::GameState, objects::ObjectType};
use de_objects::{ColliderCache, LaserCannon, ObjectCache};
use parry3d::query::Ray;

use crate::laser::LaserFireEvent;
use crate::{sightline::LineOfSight, AttackingSet};

/// Multiple of cannon range. The attacking entities will try to stay as close
/// or further from attacked targets.
const MIN_CHASE_DISTNACE: f32 = 0.4;
/// Multiple of cannon range. The attacking entities will try to stay as close
/// or closer from attacked targets.
const MAX_CHASE_DISTNACE: f32 = 0.9;

pub(crate) struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AttackEvent>()
            .add_system(
                attack
                    .in_base_set(GameSet::PreUpdate)
                    .run_if(in_state(GameState::Playing))
                    .before(ChaseSet::ChaseTargetEvent),
            )
            .add_system(
                update
                    .in_base_set(GameSet::Update)
                    .run_if(in_state(GameState::Playing))
                    .in_set(AttackingSet::Update),
            )
            .add_system(
                aim_and_fire
                    .in_base_set(GameSet::Update)
                    .run_if(in_state(GameState::Playing))
                    .after(AttackingSet::Update)
                    .before(AttackingSet::Fire),
            );
    }
}

pub struct AttackEvent {
    attacker: Entity,
    enemy: Entity,
}

impl AttackEvent {
    pub fn new(attacker: Entity, enemy: Entity) -> Self {
        Self { attacker, enemy }
    }

    fn attacker(&self) -> Entity {
        self.attacker
    }

    fn enemy(&self) -> Entity {
        self.enemy
    }
}

#[derive(Component)]
struct Attacking;

fn attack(
    mut attack_events: EventReader<AttackEvent>,
    cannons: Query<&LaserCannon>,
    mut chase_events: EventWriter<ChaseTargetEvent>,
) {
    for event in attack_events.iter() {
        if let Ok(cannon) = cannons.get(event.attacker()) {
            let target = ChaseTarget::new(
                event.enemy(),
                MIN_CHASE_DISTNACE * cannon.range(),
                MAX_CHASE_DISTNACE * cannon.range(),
            );
            chase_events.send(ChaseTargetEvent::new(event.attacker(), Some(target)));
        }
    }
}

fn update(time: Res<Time>, mut cannons: Query<&mut LaserCannon, With<Attacking>>) {
    for mut cannon in cannons.iter_mut() {
        cannon.timer_mut().tick(time.delta());
    }
}

fn aim_and_fire(
    mut commands: Commands,
    cache: Res<ObjectCache>,
    mut attackers: Query<(
        Entity,
        &Transform,
        &mut LaserCannon,
        &ChaseTargetComponent,
        Option<&Attacking>,
    )>,
    targets: Query<(&Transform, &ObjectType)>,
    sightline: LineOfSight,
    mut events: EventWriter<LaserFireEvent>,
) {
    let attackers = attackers.iter_mut();
    // The queue is used so that attacking has the same result as if it was
    // done in real-time (unaffected by update frequency).
    let mut fire_queue = BinaryHeap::new();

    for (attacker, attacker_transform, mut cannon, target, marker) in attackers {
        let target_position = match targets.get(target.target()) {
            Ok((transform, &object_type)) => {
                let centroid: Vec3 = cache.get_collider(object_type).aabb().center().into();
                transform.translation + centroid
            }
            Err(_) => continue,
        };

        let muzzle = attacker_transform.translation + cannon.muzzle();
        let to_target = (target_position - muzzle)
            .try_normalize()
            .expect("Attacker and target to close together");
        let ray = Ray::new(muzzle.into(), to_target.into());
        let aims_at_target = sightline
            .sight(&ray, cannon.range(), attacker)
            .entity()
            .map_or(true, |e| e != target.target());

        if aims_at_target {
            if marker.is_some() {
                cannon.timer_mut().reset();
                commands.entity(attacker).remove::<Attacking>();
            }
        } else {
            if marker.is_none() {
                commands.entity(attacker).insert(Attacking);
            }
            if cannon.timer_mut().check_and_update() {
                fire_queue.push(FireScheduleItem::new(attacker, ray, cannon.into_inner()));
            }
        }
    }

    while let Some(mut fire_schedule_item) = fire_queue.pop() {
        if fire_schedule_item.fire(&mut events) {
            fire_queue.push(fire_schedule_item);
        }
    }
}

struct FireScheduleItem<'a> {
    attacker: Entity,
    ray: Ray,
    cannon: &'a mut LaserCannon,
}

impl<'a> FireScheduleItem<'a> {
    fn new(attacker: Entity, ray: Ray, cannon: &'a mut LaserCannon) -> Self {
        Self {
            attacker,
            ray,
            cannon,
        }
    }

    fn fire(&mut self, events: &mut EventWriter<LaserFireEvent>) -> bool {
        events.send(LaserFireEvent::new(
            self.attacker,
            self.ray,
            self.cannon.range(),
            self.cannon.damage(),
        ));
        self.cannon.timer_mut().check_and_update()
    }
}

impl<'a> Ord for FireScheduleItem<'a> {
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

impl<'a> PartialOrd for FireScheduleItem<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> PartialEq for FireScheduleItem<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.ray.origin == other.ray.origin && self.cannon.timer() == other.cannon.timer()
    }
}

impl<'a> Eq for FireScheduleItem<'a> {}
