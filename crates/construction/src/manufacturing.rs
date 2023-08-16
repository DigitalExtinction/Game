use std::{collections::VecDeque, time::Duration};

use ahash::AHashMap;
use bevy::prelude::*;
use de_audio::spatial::{PlaySpatialAudioEvent, Sound};
use de_core::{
    cleanup::DespawnOnGameExit,
    gamestate::GameState,
    objects::{Active, ActiveObjectType, ObjectType, UnitType, PLAYER_MAX_UNITS},
    player::Player,
    projection::{ToAltitude, ToFlat},
    state::AppState,
};
use de_index::SpatialQuery;
use de_objects::SolidObjects;
use de_pathing::{PathQueryProps, PathTarget, UpdateEntityPathEvent};
use de_signs::{
    LineLocation, UpdateLineEndEvent, UpdateLineLocationEvent, UpdatePoleLocationEvent,
};
use de_spawner::{ObjectCounter, SpawnBundle};
use parry2d::bounding_volume::Aabb;
use parry3d::math::Isometry;

const MANUFACTURING_TIME: Duration = Duration::from_secs(2);
const DEFAULT_TARGET_DISTANCE: f32 = 20.;

pub(crate) struct ManufacturingPlugin;

impl Plugin for ManufacturingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnqueueAssemblyEvent>()
            .add_event::<ChangeDeliveryLocationEvent>()
            .add_event::<DeliverEvent>()
            .add_systems(
                PreUpdate,
                (
                    change_locations.in_set(ManufacturingSet::ChangeLocations),
                    check_spawn_locations.before(ManufacturingSet::Produce),
                    produce.in_set(ManufacturingSet::Produce),
                    deliver
                        .after(ManufacturingSet::ChangeLocations)
                        .after(ManufacturingSet::Produce),
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, enqueue.run_if(in_state(GameState::Playing)))
            .add_systems(PostUpdate, configure.run_if(in_state(AppState::InGame)));
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum ManufacturingSet {
    ChangeLocations,
    Produce,
}

/// Send this event to change target location of freshly manufactured units.
#[derive(Event)]
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
#[derive(Event)]
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

#[derive(Event)]
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
    blocks: Blocks,
    queue: VecDeque<ProductionItem>,
}

impl AssemblyLine {
    fn blocks_mut(&mut self) -> &mut Blocks {
        &mut self.blocks
    }

    /// Returns the first item in the assembly line (i.e. the first one to be
    /// delivered).
    fn current(&self) -> Option<UnitType> {
        self.queue.front().map(|item| item.unit())
    }

    /// Put another unit into the manufacturing queue.
    fn enqueue(&mut self, unit: UnitType, time: Duration) {
        let mut item = ProductionItem::new(unit);
        if self.queue.is_empty() {
            item.restart(time);
        }
        self.queue.push_back(item);
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
            if self.blocks.blocked() {
                self.queue.front_mut().unwrap().block(time);
                None
            } else {
                let item = self.queue.pop_front().unwrap();

                if item.is_active() {
                    if let Some(next) = self.queue.front_mut() {
                        next.restart(time - time_past);
                    }
                }

                Some(item.unit())
            }
        } else {
            None
        }
    }
}

/// When the assembly line is blocked for any reason, the last unit is produced
/// up until 100% competition but is not delivered and next unit is not
/// started.
#[derive(Default)]
struct Blocks {
    /// Whether spawn location is currently occupied.
    spawn_location: bool,
    map_capacity: bool,
}

impl Blocks {
    fn blocked(&self) -> bool {
        self.spawn_location || self.map_capacity
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

    /// If the item is already finished, stop the manufacturing and clip its
    /// due time to just now.
    fn block(&mut self, time: Duration) {
        if self.progress(time) >= MANUFACTURING_TIME {
            self.accumulated = MANUFACTURING_TIME;
            self.restarted = Some(time);
        }
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
    solids: SolidObjects,
    new: Query<(Entity, &Transform, &ObjectType), Added<Active>>,
    mut pole_events: EventWriter<UpdatePoleLocationEvent>,
    mut line_events: EventWriter<UpdateLineLocationEvent>,
) {
    for (entity, transform, &object_type) in new.iter() {
        let solid = solids.get(object_type);
        if let Some(factory) = solid.factory() {
            let start = transform.transform_point(factory.position().to_msl());
            let local_aabb = solid.ichnography().local_aabb();
            let delivery_location = DeliveryLocation::initial(local_aabb, transform);
            pole_events.send(UpdatePoleLocationEvent::new(entity, delivery_location.0));
            let end = delivery_location.0.to_msl();
            line_events.send(UpdateLineLocationEvent::new(
                entity,
                LineLocation::new(start, end),
            ));
            commands
                .entity(entity)
                .insert((AssemblyLine::default(), delivery_location));
        }
    }
}

fn change_locations(
    mut events: EventReader<ChangeDeliveryLocationEvent>,
    mut locations: Query<&mut DeliveryLocation>,
    mut pole_events: EventWriter<UpdatePoleLocationEvent>,
    mut line_events: EventWriter<UpdateLineEndEvent>,
) {
    for event in events.iter() {
        if let Ok(mut location) = locations.get_mut(event.factory()) {
            let owner = event.factory();
            location.0 = event.position();
            pole_events.send(UpdatePoleLocationEvent::new(owner, event.position()));
            let end = event.position().to_msl();
            line_events.send(UpdateLineEndEvent::new(owner, end));
        }
    }
}

fn enqueue(
    time: Res<Time>,
    mut events: EventReader<EnqueueAssemblyEvent>,
    mut lines: Query<&mut AssemblyLine>,
) {
    for event in events.iter() {
        let Ok(mut line) = lines.get_mut(event.factory()) else {
            continue;
        };
        info!(
            "Enqueueing manufacturing of {} in {:?}.",
            event.unit(),
            event.factory()
        );
        line.enqueue(event.unit(), time.elapsed());
    }
}

fn check_spawn_locations(
    solids: SolidObjects,
    space: SpatialQuery<Entity>,
    mut factories: Query<(Entity, &ObjectType, &Transform, &mut AssemblyLine)>,
) {
    for (entity, &object_type, transform, mut line) in factories.iter_mut() {
        line.blocks_mut().spawn_location = match line.current() {
            Some(unit_type) => {
                let factory = solids.get(object_type).factory().unwrap();
                let collider = solids
                    .get(ObjectType::Active(ActiveObjectType::Unit(unit_type)))
                    .collider();

                let spawn_point = transform.transform_point(factory.position().to_msl());
                let isometry = Isometry::translation(spawn_point.x, spawn_point.y, spawn_point.z);
                let mut aabb = collider.aabb().transform_by(&isometry);
                aabb.mins.y = f32::NEG_INFINITY;
                aabb.maxs.y = f32::INFINITY;

                space.query_aabb(&aabb, Some(entity)).next().is_some()
            }
            None => false,
        };
    }
}

fn produce(
    time: Res<Time>,
    counter: Res<ObjectCounter>,
    mut factories: Query<(Entity, &Player, &mut AssemblyLine)>,
    mut deliver_events: EventWriter<DeliverEvent>,
) {
    let mut counts: AHashMap<Player, u32> = AHashMap::from_iter(
        counter
            .counters()
            .map(|(&player, counter)| (player, counter.unit_count())),
    );

    for (factory, &player, mut assembly) in factories.iter_mut() {
        let player_count = counts.entry(player).or_default();

        loop {
            assembly.blocks_mut().map_capacity = *player_count >= PLAYER_MAX_UNITS;

            let Some(unit_type) = assembly.produce(time.elapsed()) else {
                break;
            };
            *player_count += 1;

            deliver_events.send(DeliverEvent::new(factory, unit_type));
        }
    }
}

fn deliver(
    mut commands: Commands,
    solids: SolidObjects,
    mut deliver_events: EventReader<DeliverEvent>,
    mut path_events: EventWriter<UpdateEntityPathEvent>,
    factories: Query<(&Transform, &ObjectType, &Player, &DeliveryLocation)>,
    mut play_audio: EventWriter<PlaySpatialAudioEvent>,
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

        let factory = solids.get(factory_object_type).factory().unwrap();
        debug_assert!(factory.products().contains(&delivery.unit()));
        let spawn_point = transform.transform_point(factory.position().to_msl());

        let unit = commands
            .spawn((
                SpawnBundle::new(unit_object_type, Transform::from_translation(spawn_point)),
                player,
                DespawnOnGameExit,
            ))
            .id();
        path_events.send(UpdateEntityPathEvent::new(
            unit,
            PathTarget::new(
                delivery_location.0,
                PathQueryProps::new(0., f32::INFINITY),
                false,
            ),
        ));

        play_audio.send(PlaySpatialAudioEvent::new(Sound::Manufacture, spawn_point));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assembly_line() {
        let mut line = AssemblyLine::default();

        assert!(line.produce(Duration::from_secs(20)).is_none());
        line.enqueue(UnitType::Attacker, Duration::from_secs(21));
        line.enqueue(UnitType::Attacker, Duration::from_secs(21));

        assert!(line.produce(Duration::from_secs(22)).is_none());
        line.blocks_mut().map_capacity = true;
        assert!(line.produce(Duration::from_secs(25)).is_none());
        line.blocks_mut().map_capacity = false;
        assert_eq!(
            line.produce(Duration::from_secs(26)).unwrap(),
            UnitType::Attacker
        );
        assert!(line.produce(Duration::from_secs(26)).is_none());
        assert_eq!(
            line.produce(Duration::from_secs(27)).unwrap(),
            UnitType::Attacker
        );
        assert!(line.produce(Duration::from_secs(30)).is_none());

        line.enqueue(UnitType::Attacker, Duration::from_secs(50));
        line.enqueue(UnitType::Attacker, Duration::from_secs(51));

        assert!(line.produce(Duration::from_secs(51)).is_none());
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
