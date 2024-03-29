use std::marker::PhantomData;

use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;
use de_audio::spatial::{PlaySpatialAudioEvent, Sound};
use de_core::gconfig::GameConfig;
use de_core::{objects::ObjectTypeComponent, player::PlayerComponent, state::AppState};
use de_messages::ToPlayers;
use de_multiplayer::{
    NetEntities, NetEntityCommands, NetRecvDespawnActiveEvent, PeerLeftEvent, ToPlayersEvent,
};
use de_types::objects::{ActiveObjectType, ObjectType};

use crate::{ObjectCounter, SpawnerSet};

pub(crate) struct DespawnerPlugin;

impl Plugin for DespawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                despawn_active_local.before(despawn_active),
                despawn_active_remote
                    .run_if(on_event::<NetRecvDespawnActiveEvent>())
                    .before(despawn_active),
                despawn_active_peer_left
                    .run_if(on_event::<PeerLeftEvent>())
                    .after(despawn_active_remote)
                    .before(despawn_active),
                despawn_active.before(despawn),
                despawn,
            )
                .run_if(in_state(AppState::InGame))
                .in_set(DespawnerSet::Despawn)
                .after(SpawnerSet::Spawner),
        )
        .add_event::<DespawnActiveLocalEvent>()
        .add_event::<DespawnActiveEvent>()
        .add_event::<DespawnEvent>();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum DespawnerSet {
    /// Despawn systems are part of this set.
    Despawn,
    /// Despawn related events are send from systems of this set.
    Events,
}

#[derive(Event)]
pub struct DespawnActiveLocalEvent(Entity);

impl DespawnActiveLocalEvent {
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }
}

#[derive(Event)]
struct DespawnActiveEvent(Entity);

#[derive(Event)]
struct DespawnEvent(Entity);

fn despawn_active_local(
    config: Res<GameConfig>,
    net_entities: NetEntities,
    mut event_reader: EventReader<DespawnActiveLocalEvent>,
    mut event_writer: EventWriter<DespawnActiveEvent>,
    mut net_events: EventWriter<ToPlayersEvent>,
) {
    for event in event_reader.read() {
        event_writer.send(DespawnActiveEvent(event.0));

        if config.multiplayer() {
            net_events.send(ToPlayersEvent::new(ToPlayers::Despawn {
                entity: net_entities.local_net_id(event.0),
            }));
        }
    }
}

fn despawn_active_remote(
    mut event_reader: EventReader<NetRecvDespawnActiveEvent>,
    mut event_writer: EventWriter<DespawnActiveEvent>,
) {
    for event in event_reader.read() {
        event_writer.send(DespawnActiveEvent(event.entity()));
    }
}

fn despawn_active_peer_left(
    mut net_commands: NetEntityCommands,
    mut peer_left_events: EventReader<PeerLeftEvent>,
    mut event_writer: EventWriter<DespawnActiveEvent>,
) {
    for event in peer_left_events.read() {
        if let Some(entity_map) = net_commands.remove_player(event.id()) {
            for entity in entity_map.locals() {
                event_writer.send(DespawnActiveEvent(entity));
            }
        }
    }
}

fn despawn_active(
    mut counter: ResMut<ObjectCounter>,
    entities: Query<(&PlayerComponent, &ObjectTypeComponent, &Transform)>,
    mut event_reader: EventReader<DespawnActiveEvent>,
    mut event_writer: EventWriter<DespawnEvent>,
    mut play_audio: EventWriter<PlaySpatialAudioEvent>,
) {
    for event in event_reader.read() {
        let Ok((&player, &object_type, transform)) = entities.get(event.0) else {
            panic!("Despawn of non-existing active object requested.");
        };

        let ObjectType::Active(active_type) = *object_type else {
            panic!("Non-active object cannot be despawned with DespawnActiveEvent.");
        };

        counter.player_mut(*player).update(active_type, -1);
        play_audio.send(PlaySpatialAudioEvent::new(
            match active_type {
                ActiveObjectType::Building(_) => Sound::DestroyBuilding,
                ActiveObjectType::Unit(_) => Sound::DestroyUnit,
            },
            transform.translation,
        ));

        event_writer.send(DespawnEvent(event.0));
    }
}

/// Despawn all entities marked for despawning
fn despawn(mut commands: Commands, mut despawning: EventReader<DespawnEvent>) {
    for entity in despawning.read() {
        commands.entity(entity.0).despawn_recursive();
    }
}

type DespData<T> = <T as ToOwned>::Owned;

/// This plugin sends events with data of type `DespData<T>` when entities with
/// component `T` matching query `F` are despawned. The events are send from
/// systems in set [`DespawnerSet::Events`].
///
/// # Type Parameters
///
/// * `T` - a component implementing ToOwned to be send as part of the despawn
///   events.
/// * `F` - filter for entities to watch for despawning. (optional, defaults to
///   `()`). e.g. `With<Bar>`.
///
/// # Usage
///
/// ```
/// use bevy::prelude::*;
/// use de_spawner::DespawnEventsPlugin;
///
/// #[derive(Clone, Component)] // we must Clone implement here
/// struct Bar(f32);
///
/// let mut app = App::new();
///
/// // watch for despawning of entities with `Bar` component
/// app.add_plugins(DespawnEventsPlugin::<Bar>::default());
/// ```
///
#[derive(Debug)]
pub struct DespawnEventsPlugin<T, F = ()>
where
    T: Component + ToOwned,
    DespData<T>: Send + Sync,
    F: QueryFilter + Send + Sync + 'static,
{
    _marker: PhantomData<(T, F)>,
}

impl<T, F> Default for DespawnEventsPlugin<T, F>
where
    T: Component + ToOwned,
    DespData<T>: Send + Sync,
    F: QueryFilter + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T, F> Plugin for DespawnEventsPlugin<T, F>
where
    T: Component + ToOwned,
    DespData<T>: Send + Sync,
    F: QueryFilter + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_event::<DespawnedComponentsEvent<DespData<T>>>()
            .add_systems(
                Update,
                send_data::<T, F>
                    .after(DespawnerSet::Despawn)
                    .in_set(DespawnerSet::Events),
            );
    }
}

/// This event is sent by [`DespawnEventsPlugin`] when a matching entity is
/// being despawned.
#[derive(Event)]
pub struct DespawnedComponentsEvent<D>
where
    D: Send + Sync,
{
    pub entity: Entity,
    pub data: D,
}

/// Sends events with data of type `DespData<T>` when entities with component
/// `T` and matching query `F` are despawned.
#[allow(unused)]
fn send_data<T, F>(
    mut despawning: EventReader<DespawnEvent>,
    mut events: EventWriter<DespawnedComponentsEvent<DespData<T>>>,
    data: Query<&T, F>,
) where
    T: Component + ToOwned,
    DespData<T>: Send + Sync,
    F: QueryFilter + Send + Sync,
{
    for DespawnEvent(entity) in despawning.read() {
        if let Ok(data) = data.get(*entity) {
            events.send(DespawnedComponentsEvent {
                entity: *entity,
                data: data.to_owned(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::{
        ecs::{schedule::ScheduleBuildSettings, system::SystemState},
        log::{Level, LogPlugin},
    };

    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, Component)]
    struct TestComponent {
        pub value: usize,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Component)]
    struct ComplexComponent {
        pub value: usize,
        pub value2: ComplexStruct,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct ComplexStruct {
        pub foo: usize,
        pub bar: String,
    }

    fn despawn_all_test_system(
        query: Query<Entity, With<TestComponent>>,
        mut event_writer: EventWriter<DespawnEvent>,
    ) {
        for entity in query.iter() {
            trace!("Entity queued for despawning -> {:?}", entity);
            event_writer.send(DespawnEvent(entity));
        }
    }

    #[test]
    fn despawn_events() {
        let mut app = App::new();
        app.add_plugins(LogPlugin {
            level: Level::TRACE,
            ..Default::default()
        });

        let simple_entity = app.world.spawn((TestComponent { value: 1 },)).id();
        trace!("Simple entity spawned -> {:?}", simple_entity);

        app.edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                auto_insert_apply_deferred: false,
                ..default()
            });
        })
        .add_plugins(DespawnEventsPlugin::<TestComponent>::default())
        .add_plugins(DespawnEventsPlugin::<ComplexComponent, With<TestComponent>>::default())
        .add_systems(
            Update,
            (despawn_all_test_system.before(DespawnerSet::Despawn),),
        )
        .add_systems(Update, despawn.in_set(DespawnerSet::Despawn))
        .add_event::<DespawnEvent>();

        let mut simple_events =
            SystemState::<EventReader<DespawnedComponentsEvent<TestComponent>>>::new(
                &mut app.world,
            );
        let mut complex_events = SystemState::<
            EventReader<DespawnedComponentsEvent<ComplexComponent>>,
        >::new(&mut app.world);

        trace!("---------- App update #1 ----------");
        app.update();
        trace!("-----------------------------------");

        assert_eq!(
            simple_events.get(&app.world).read().next().unwrap().data,
            TestComponent { value: 1 }
        );

        let complex_entity = app
            .world
            .spawn((
                TestComponent { value: 1 },
                ComplexComponent {
                    value: 2,
                    value2: ComplexStruct {
                        foo: 3,
                        bar: "Hello World".to_string(),
                    },
                },
            ))
            .id();
        trace!("Complex entity spawned -> {:?}", complex_entity);

        trace!("---------- App update #2 ----------");
        app.update();
        trace!("-----------------------------------");

        assert_eq!(
            complex_events.get(&app.world).read().next().unwrap().data,
            ComplexComponent {
                value: 2,
                value2: ComplexStruct {
                    foo: 3,
                    bar: "Hello World".to_string(),
                },
            }
        );

        trace!("---------- App update #3 ----------");
        app.update(); // nothing should happen
        trace!("-----------------------------------");
    }
}
