use std::marker::PhantomData;

use bevy::ecs::query::{ReadOnlyWorldQuery, WorldQuery};
use bevy::prelude::*;
use de_audio::spatial::{PlaySpatialAudioEvent, Sound};
use de_core::gconfig::GameConfig;
use de_core::{
    objects::{Local, ObjectTypeComponent},
    player::PlayerComponent,
    state::AppState,
};
use de_messages::ToPlayers;
use de_multiplayer::{NetRecvDespawnActiveEvent, ToPlayersEvent};
use de_objects::Health;
use de_types::objects::{ActiveObjectType, ObjectType};

use crate::ObjectCounter;

pub(crate) struct DespawnerPlugin;

impl Plugin for DespawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                find_dead
                    .in_set(DespawnerSet::Destruction)
                    .before(despawn_active_local),
                despawn_active_local.before(despawn_active),
                despawn_active_remote
                    .run_if(on_event::<NetRecvDespawnActiveEvent>())
                    .before(despawn_active),
                despawn_active.before(despawn),
                despawn.in_set(DespawnerSet::Despawn),
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_event::<DespawnActiveLocalEvent>()
        .add_event::<DespawnActiveEvent>()
        .add_event::<DespawnEvent>();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum DespawnerSet {
    /// This set is run before the despawning systems run (If you expect units to die, you should
    /// run your system in this set and the [`Update`] base set)
    Destruction,
    /// All despawn logic is anchored on this (You might want to run your system after this to get
    /// [`DespawnedComponentsEvent`]s)
    Despawn,
}

#[derive(Event)]
struct DespawnActiveLocalEvent(Entity);

#[derive(Event)]
struct DespawnActiveEvent(Entity);

#[derive(Event)]
struct DespawnEvent(Entity);

type LocallyChangedHealth<'w, 's> =
    Query<'w, 's, (Entity, &'static Health), (With<Local>, Changed<Health>)>;

/// Find all entities with low health and mark them for despawning
fn find_dead(
    entities: LocallyChangedHealth,
    mut event_writer: EventWriter<DespawnActiveLocalEvent>,
) {
    for (entity, health) in entities.iter() {
        if health.destroyed() {
            event_writer.send(DespawnActiveLocalEvent(entity));
        }
    }
}

fn despawn_active_local(
    config: Res<GameConfig>,
    mut event_reader: EventReader<DespawnActiveLocalEvent>,
    mut event_writer: EventWriter<DespawnActiveEvent>,
    mut net_events: EventWriter<ToPlayersEvent>,
) {
    for event in event_reader.iter() {
        event_writer.send(DespawnActiveEvent(event.0));

        if config.multiplayer() {
            net_events.send(ToPlayersEvent::new(ToPlayers::Despawn {
                entity: event.0.into(),
            }));
        }
    }
}

fn despawn_active_remote(
    mut event_reader: EventReader<NetRecvDespawnActiveEvent>,
    mut event_writer: EventWriter<DespawnActiveEvent>,
) {
    for event in event_reader.iter() {
        event_writer.send(DespawnActiveEvent(event.entity()));
    }
}

fn despawn_active(
    mut counter: ResMut<ObjectCounter>,
    entities: Query<(&PlayerComponent, &ObjectTypeComponent, &Transform), Changed<Health>>,
    mut event_reader: EventReader<DespawnActiveEvent>,
    mut event_writer: EventWriter<DespawnEvent>,
    mut play_audio: EventWriter<PlaySpatialAudioEvent>,
) {
    for event in event_reader.iter() {
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
    for entity in despawning.iter() {
        commands.entity(entity.0).despawn_recursive();
    }
}

/// This plugin sends events with data of type `T`when entities with `Q` and matching `F` are despawned.
///
/// # Type Parameters
///
/// * `Q` - query for entities to watch for despawning. e.g `&Foo`.
/// * `T` - type of data to send (must be a single component contained in `Q`). e.g. `Foo`.
/// * `F` - filter for entities to watch for despawning. (optional, defaults to `()`). e.g. `With<Bar>`.
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
/// app.add_plugins(DespawnEventsPlugin::<&Bar, Bar>::default());
/// ```
///
#[derive(Debug)]
pub struct DespawnEventsPlugin<Q, T, F = ()>
where
    F: ReadOnlyWorldQuery + Send + Sync + 'static,
    T: Send + Sync + 'static + Clone + Component,
    Q: WorldQuery + Send + Sync + 'static,
{
    _marker: PhantomData<(Q, F, T)>,
}

impl<
        Q: WorldQuery + Send + Sync + 'static,
        T: Send + Sync + 'static + Clone + Component,
        F: ReadOnlyWorldQuery + Send + Sync + 'static,
    > Default for DespawnEventsPlugin<Q, T, F>
{
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<
        Q: WorldQuery + Send + Sync + 'static,
        T: Send + Sync + Clone + Component + 'static,
        F: ReadOnlyWorldQuery + Send + Sync + 'static,
    > Plugin for DespawnEventsPlugin<Q, T, F>
{
    fn build(&self, app: &mut App) {
        app.add_event::<DespawnedComponentsEvent<T, F>>()
            .add_systems(
                Update,
                (send_data::<Q, T, F>
                    .after(DespawnerSet::Destruction)
                    .in_set(DespawnerSet::Despawn),),
            );
    }
}

/// This event is sent by [`DespawnEventsPlugin`] when a matching entity is being despawned.
#[derive(Debug, Event)]
pub struct DespawnedComponentsEvent<T, F = ()>
where
    T: Send + Sync + Component + 'static,
    F: ReadOnlyWorldQuery + 'static,
{
    pub entity: Entity,
    pub data: T,
    _filter_marker: PhantomData<F>,
}

/// Send events with data of type `T` when entities with `Q` and matching `F` are despawned.
#[allow(unused)]
fn send_data<'w, Q, T, F>(
    mut despawning: EventReader<DespawnEvent>,
    mut events: EventWriter<DespawnedComponentsEvent<T, F>>,
    data: Query<Q, F>,
) where
    T: Clone + Component + Send + Sync + 'w,
    Q: WorldQuery + Send + Sync + 'w,
    F: ReadOnlyWorldQuery + Send + Sync + 'w,
{
    for DespawnEvent(entity) in despawning.iter() {
        if let Ok(data) = data.get_component::<T>(*entity) {
            events.send(DespawnedComponentsEvent {
                entity: *entity,
                data: data.clone(),
                _filter_marker: PhantomData,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::SystemState;
    use bevy::log::{Level, LogPlugin};

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

        app.add_plugins(DespawnEventsPlugin::<&TestComponent, TestComponent>::default())
            .add_plugins(DespawnEventsPlugin::<
                &ComplexComponent,
                ComplexComponent,
                With<TestComponent>,
            >::default())
            .add_systems(
                Update,
                (despawn_all_test_system.in_set(DespawnerSet::Destruction),),
            )
            .add_systems(
                Update,
                despawn
                    .in_set(DespawnerSet::Despawn)
                    .after(DespawnerSet::Destruction),
            )
            .add_event::<DespawnEvent>();

        let mut simple_events =
            SystemState::<EventReader<DespawnedComponentsEvent<TestComponent>>>::new(
                &mut app.world,
            );
        let mut complex_events = SystemState::<
            EventReader<DespawnedComponentsEvent<ComplexComponent, With<TestComponent>>>,
        >::new(&mut app.world);

        trace!("---------- App update #1 ----------");
        app.update();
        trace!("-----------------------------------");

        assert_eq!(
            simple_events.get(&app.world).iter().next().unwrap().data,
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
            complex_events.get(&app.world).iter().next().unwrap().data,
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
