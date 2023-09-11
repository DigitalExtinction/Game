use std::{cmp::Ordering, collections::BinaryHeap};

use bevy::prelude::*;
use de_behaviour::{ChaseSet, ChaseTarget, ChaseTargetEvent};
use de_core::{gamestate::GameState, objects::ObjectTypeComponent};
use de_index::SpatialQuery;
use de_objects::{LaserCannon, SolidObjects};
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
            .add_systems(
                PreUpdate,
                (
                    attack
                        .in_set(AttackingSet::Attack)
                        .before(ChaseSet::ChaseTargetEvent),
                    update_positions.after(AttackingSet::Attack),
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    charge.in_set(AttackingSet::Charge),
                    aim_and_fire
                        .after(AttackingSet::Charge)
                        .before(AttackingSet::Fire),
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Event)]
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
struct Attacking {
    enemy: Entity,
    muzzle: Vec3,
    target: Option<Vec3>,
}

impl Attacking {
    fn new(enemy: Entity) -> Self {
        Self {
            enemy,
            muzzle: Vec3::ZERO,
            target: None,
        }
    }

    fn distance(&self) -> Option<f32> {
        self.target.map(|target| target.distance(self.muzzle))
    }

    fn ray(&self) -> Option<Ray> {
        self.target.map(|target| {
            let direction = (target - self.muzzle).normalize();
            Ray::new(self.muzzle.into(), direction.into())
        })
    }
}

fn attack(
    mut commands: Commands,
    mut attack_events: EventReader<AttackEvent>,
    cannons: Query<&LaserCannon>,
    mut chase_events: EventWriter<ChaseTargetEvent>,
) {
    for event in attack_events.iter() {
        if let Ok(cannon) = cannons.get(event.attacker()) {
            commands
                .entity(event.attacker())
                .insert(Attacking::new(event.enemy()));

            let target = ChaseTarget::new(
                event.enemy(),
                MIN_CHASE_DISTNACE * cannon.range(),
                MAX_CHASE_DISTNACE * cannon.range(),
            );
            chase_events.send(ChaseTargetEvent::new(event.attacker(), Some(target)));
        }
    }
}

fn update_positions(
    mut commands: Commands,
    solids: SolidObjects,
    mut cannons: Query<(Entity, &Transform, &LaserCannon, &mut Attacking)>,
    targets: Query<(&Transform, &ObjectTypeComponent)>,
    sightline: SpatialQuery<Entity>,
) {
    for (attacker, transform, cannon, mut attacking) in cannons.iter_mut() {
        match targets.get(attacking.enemy) {
            Ok((enemy_transform, &target_type)) => {
                attacking.muzzle = transform.translation + cannon.muzzle();

                let enemy_aabb = solids.get(*target_type).collider().aabb();
                let enemy_centroid = enemy_transform.translation + Vec3::from(enemy_aabb.center());
                let direction = (enemy_centroid - attacking.muzzle)
                    .try_normalize()
                    .expect("Attacker and target too close together");
                let cannon_ray = Ray::new(attacking.muzzle.into(), direction.into());

                attacking.target = sightline
                    .cast_ray(&cannon_ray, cannon.range(), Some(attacker))
                    .map(|intersection| cannon_ray.point_at(intersection.toi()).into());
            }
            Err(_) => {
                commands.entity(attacker).remove::<Attacking>();
            }
        }
    }
}

fn charge(time: Res<Time>, mut cannons: Query<(&mut LaserCannon, Option<&Attacking>)>) {
    for (mut cannon, attacking) in cannons.iter_mut() {
        let charge = attacking
            .and_then(|attacking| attacking.distance())
            .map_or(false, |distance| distance <= cannon.range());
        cannon.charge_mut().tick(time.delta(), charge);
    }
}

fn aim_and_fire(
    mut attackers: Query<(Entity, &mut LaserCannon, &Attacking)>,
    sightline: LineOfSight,
    mut events: EventWriter<LaserFireEvent>,
) {
    let attackers = attackers.iter_mut();
    // The queue is used so that attacking has the same result as if it was
    // done in real-time (unaffected by update frequency).
    let mut fire_queue = BinaryHeap::new();

    for (attacker, mut cannon, attacking) in attackers {
        let ray = attacking.ray().filter(|ray| {
            sightline
                .sight(ray, cannon.range(), attacker)
                .entity()
                .map_or(false, |e| e == attacking.enemy)
        });

        if let Some(ray) = ray {
            if cannon.charge().charged() {
                fire_queue.push(FireScheduleItem::new(attacker, ray, cannon.into_inner()));
            }
        } else {
            cannon.charge_mut().hold();
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
        self.cannon.charge_mut().fire()
    }
}

impl<'a> Ord for FireScheduleItem<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        let ordering = self.cannon.charge().cmp(other.cannon.charge());
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
        self.ray.origin == other.ray.origin && self.cannon.charge() == other.cannon.charge()
    }
}

impl<'a> Eq for FireScheduleItem<'a> {}
