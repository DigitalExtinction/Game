use std::{collections::VecDeque, time::Duration};

use ahash::AHashMap;
use bevy::prelude::*;
use de_core::{
    baseset::GameSet,
    cleanup::DespawnOnGameExit,
    gamestate::GameState,
    gconfig::GameConfig,
    objects::{ActiveObjectType, ObjectType, UnitType, PLAYER_MAX_UNITS},
    player::Player,
    projection::{ToAltitude, ToFlat},
    state::AppState,
};
use de_objects::{IchnographyCache, ObjectCache};
use de_pathing::{PathQueryProps, PathTarget, UpdateEntityPath};
use de_spawner::{ObjectCounter, SpawnBundle};
use parry2d::bounding_volume::Aabb;

const MANUFACTURING_TIME: Duration = Duration::from_secs(2);
const DEFAULT_TARGET_DISTANCE: f32 = 20.;

pub(crate) struct ManufacturingPlugin;

impl Plugin for ManufacturingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnqueueAssemblyEvent>()
            .add_event::<ChangeDeliveryLocationEvent>()
            .add_event::<DeliverEvent>()
            .add_system(
                configure
                    .in_base_set(GameSet::PostUpdate)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_system(
                change_locations
                    .in_base_set(GameSet::PreUpdate)
                    .run_if(in_state(GameState::Playing))
                    .in_set(ManufacturingSet::ChangeLocations),
            )
            .add_system(
                enqueue
                    .in_base_set(GameSet::Update)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_system(
                produce
                    .in_base_set(GameSet::PreUpdate)
                    .run_if(in_state(GameState::Playing))
                    .in_set(ManufacturingSet::Produce),
            )
            .add_system(
                deliver
                    .in_base_set(GameSet::PreUpdate)
                    .run_if(in_state(GameState::Playing))
                    .after(ManufacturingSet::ChangeLocations)
                    .after(ManufacturingSet::Produce),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum ManufacturingSet {
    ChangeLocations,
    Produce,
}

/// Send this event to change target location of freshly manufactured units.
pub struct ChangeDeliveryLocationEvent {
    factory: Entity,
    position: Vec2,
}

impl ChangeDeliveryLocationEvent {
    pub fn new(factory: Entity, position: Vec2) -> Self {
        Self { factory, position }
    }

    fn factory(&self) -> Entity {
        self.factory
    }

    fn position(&self) -> Vec2 {
        self.position
    }
}

/// Send this event to enqueue a unit to be manufactured by a factory.
pub struct EnqueueAssemblyEvent {
    factory: Entity,
    unit: UnitType,
}

impl EnqueueAssemblyEvent {
    /// # Arguments
    ///
    /// `factory` - the building to produce the unit.
    ///
    /// `unit` - unit to be produced.
    pub fn new(factory: Entity, unit: UnitType) -> Self {
        Self { factory, unit }
    }

    fn factory(&self) -> Entity {
        self.factory
    }

    fn unit(&self) -> UnitType {
        self.unit
    }
}

struct DeliverEvent {
    factory: Entity,
    unit: UnitType,
}

impl DeliverEvent {
    fn new(factory: Entity, unit: UnitType) -> Self {
        Self { factory, unit }
    }

    fn factory(&self) -> Entity {
        self.factory
    }

    fn unit(&self) -> UnitType {
        self.unit
    }
}

#[derive(Component)]
struct DeliveryLocation(Vec2);

impl DeliveryLocation {
    fn initial(local_aabb: Aabb, transform: &Transform) -> Self {
        let target = Vec2::new(
            local_aabb.maxs.x + DEFAULT_TARGET_DISTANCE,
            0.5 * (local_aabb.mins.y + local_aabb.maxs.y),
        );
        Self(transform.transform_point(target.to_msl()).to_flat())
    }
}

/// An assembly line attached to every building and capable of production of
/// any units.
#[derive(Component, Default)]
pub struct AssemblyLine {
    queue: VecDeque<ProductionItem>,
}

impl AssemblyLine {
    /// Put another unit into the manufacturing queue.
    fn enqueue(&mut self, unit: UnitType) {
        self.queue.push_back(ProductionItem::new(unit));
    }

    /// In case the assembly line is stopped, restart the production.
    ///
    /// # Arguments
    ///
    /// * `time` - elapsed time since a fixed point in time in the past.
    fn restart(&mut self, time: Duration) {
        if let Some(item) = self.queue.front_mut().filter(|item| !item.is_active()) {
            item.restart(time);
        }
    }

    /// In case the assembly line is actively manufacturing some units, stop
    /// it.
    ///
    /// # Arguments
    ///
    /// * `time` - elapsed time since a fixed point in time in the past.
    fn stop(&mut self, time: Duration) {
        if let Some(item) = self.queue.front_mut().filter(|item| item.is_active()) {
            item.stop(time);
        }
    }

    /// Update the production line.
    ///
    /// This method should be called repeatedly and during every tick until it
    /// returns None. The returned values correspond to finished units.
    ///
    /// # Arguments
    ///
    /// * `time` - elapsed time since a fixed point in time in the past.
    fn produce(&mut self, time: Duration) -> Option<UnitType> {
        if let Some(time_past) = self.queue.front().and_then(|item| item.finished(time)) {
            let item = self.queue.pop_front().unwrap();

            if item.is_active() {
                if let Some(next) = self.queue.front_mut() {
                    next.restart(time - time_past);
                }
            }

            Some(item.unit())
        } else {
            None
        }
    }
}

/// A single unit being manufactured / enqueued for manufacturing in an
/// assembly line.
struct ProductionItem {
    /// Total accumulated production time of the item until the last
    /// stop/restart to the manufacturing.
    accumulated: Duration,
    /// Time elapsed since a fixed point in the past until when manufacturing
    /// of the unit was restarted for the last time.
    restarted: Option<Duration>,
    unit: UnitType,
}

impl ProductionItem {
    fn new(unit: UnitType) -> Self {
        Self {
            accumulated: Duration::ZERO,
            restarted: None,
            unit,
        }
    }

    fn unit(&self) -> UnitType {
        self.unit
    }

    /// Returns true if the unit is actively manufactured.
    fn is_active(&self) -> bool {
        self.restarted.is_some()
    }

    /// Restarts (stops and starts) manufacturing of the unit.
    fn restart(&mut self, time: Duration) {
        self.stop(time);
        self.restarted = Some(time);
    }

    /// Stops manufacturing of the unit if it is currently being manufactured.
    ///
    /// Total accumulated manufacturing time is clipped to the time it takes to
    /// produce the unit.
    fn stop(&mut self, time: Duration) {
        if let Some(last) = self.restarted {
            self.accumulated += time - last;
            if self.accumulated > MANUFACTURING_TIME {
                self.accumulated = MANUFACTURING_TIME;
            }
        }
        self.restarted = None;
    }

    /// Returns None if the unit is not yet finished. Otherwise, it returns for
    /// how long it has been finished.
    fn finished(&self, time: Duration) -> Option<Duration> {
        let progress = self.progress(time);
        if progress >= MANUFACTURING_TIME {
            Some(progress - MANUFACTURING_TIME)
        } else {
            None
        }
    }

    /// Returns for how long cumulatively the unit has been manufactured.
    fn progress(&self, time: Duration) -> Duration {
        self.accumulated
            + self
                .restarted
                .map_or(Duration::ZERO, |restarted| time - restarted)
    }
}

fn configure(
    mut commands: Commands,
    cache: Res<ObjectCache>,
    new: Query<(Entity, &Transform, &ObjectType), Added<ObjectType>>,
) {
    for (entity, transform, &object_type) in new.iter() {
        if cache.get(object_type).factory().is_some() {
            let local_aabb = cache.get_ichnography(object_type).local_aabb();
            let delivery_location = DeliveryLocation::initial(local_aabb, transform);
            commands
                .entity(entity)
                .insert((AssemblyLine::default(), delivery_location));
        }
    }
}

fn change_locations(
    mut events: EventReader<ChangeDeliveryLocationEvent>,
    mut locations: Query<&mut DeliveryLocation>,
) {
    for event in events.iter() {
        if let Ok(mut location) = locations.get_mut(event.factory()) {
            location.0 = event.position();
        }
    }
}

fn enqueue(mut events: EventReader<EnqueueAssemblyEvent>, mut lines: Query<&mut AssemblyLine>) {
    for event in events.iter() {
        let Ok(mut line) = lines.get_mut(event.factory()) else { continue };
        info!(
            "Enqueueing manufacturing of {} in {:?}.",
            event.unit(),
            event.factory()
        );
        line.enqueue(event.unit());
    }
}

fn produce(
    time: Res<Time>,
    conf: Res<GameConfig>,
    counter: Res<ObjectCounter>,
    mut factories: Query<(Entity, &Player, &mut AssemblyLine)>,
    mut deliver_events: EventWriter<DeliverEvent>,
) {
    let mut counts: AHashMap<Player, u32> = AHashMap::new();
    for player in conf.players() {
        let count = counter.player(player).unwrap().unit_count();
        counts.insert(player, count);
    }

    for (factory, &player, mut assembly) in factories.iter_mut() {
        let player_count = counts.get_mut(&player).unwrap();
        if *player_count < PLAYER_MAX_UNITS {
            assembly.restart(time.elapsed());
        }

        loop {
            if *player_count >= PLAYER_MAX_UNITS {
                assembly.stop(time.elapsed());
                break;
            }

            let Some(unit_type) = assembly.produce(time.elapsed()) else { break };
            *player_count += 1;

            deliver_events.send(DeliverEvent::new(factory, unit_type));
        }
    }
}

fn deliver(
    mut commands: Commands,
    cache: Res<ObjectCache>,
    mut deliver_events: EventReader<DeliverEvent>,
    mut path_events: EventWriter<UpdateEntityPath>,
    factories: Query<(&Transform, &ObjectType, &Player, &DeliveryLocation)>,
) {
    for delivery in deliver_events.iter() {
        info!(
            "Manufacturing of {} in {:?} just finished.",
            delivery.unit(),
            delivery.factory()
        );

        let (transform, &factory_object_type, &player, delivery_location) =
            factories.get(delivery.factory()).unwrap();
        let unit_object_type = ObjectType::Active(ActiveObjectType::Unit(delivery.unit()));

        let factory = cache.get(factory_object_type).factory().unwrap();
        debug_assert!(factory.products().contains(&delivery.unit()));
        let spawn_point = transform.transform_point(factory.position().to_msl());

        let unit = commands
            .spawn((
                SpawnBundle::new(unit_object_type, Transform::from_translation(spawn_point)),
                player,
                DespawnOnGameExit,
            ))
            .id();
        path_events.send(UpdateEntityPath::new(
            unit,
            PathTarget::new(
                delivery_location.0,
                PathQueryProps::new(0., f32::INFINITY),
                false,
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assembly_line() {
        let mut line = AssemblyLine::default();

        line.restart(Duration::from_secs(0));
        assert!(line.produce(Duration::from_secs(20)).is_none());

        line.enqueue(UnitType::Attacker);
        line.enqueue(UnitType::Attacker);
        line.restart(Duration::from_secs(20));
        assert!(line.produce(Duration::from_secs(21)).is_none());
        assert_eq!(
            line.produce(Duration::from_secs(23)).unwrap(),
            UnitType::Attacker
        );
        assert!(line.produce(Duration::from_secs(23)).is_none());
        assert_eq!(
            line.produce(Duration::from_secs(24)).unwrap(),
            UnitType::Attacker
        );
        assert!(line.produce(Duration::from_secs(30)).is_none());

        line.enqueue(UnitType::Attacker);
        line.enqueue(UnitType::Attacker);
        line.restart(Duration::from_secs(50));
        assert!(line.produce(Duration::from_secs(51)).is_none());
        line.stop(Duration::from_secs(51));
        line.restart(Duration::from_secs(60));
        assert!(line.produce(Duration::from_secs_f32(60.5)).is_none());
        assert_eq!(
            line.produce(Duration::from_secs(61)).unwrap(),
            UnitType::Attacker
        );
        assert_eq!(
            line.produce(Duration::from_secs(63)).unwrap(),
            UnitType::Attacker
        );
        assert!(line.produce(Duration::from_secs(90)).is_none());
    }
}
