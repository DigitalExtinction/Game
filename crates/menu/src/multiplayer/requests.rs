use std::marker::PhantomData;

use bevy::{ecs::system::SystemParam, prelude::*};
use de_lobby_client::{LobbyRequest, RequestEvent, ResponseEvent, Result};

use crate::MenuState;

pub(super) struct RequestsPlugin<T>
where
    T: LobbyRequest,
{
    _marker: PhantomData<fn() -> T>,
}

impl<T> RequestsPlugin<T>
where
    T: LobbyRequest,
{
    pub(super) fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Plugin for RequestsPlugin<T>
where
    T: LobbyRequest,
{
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::Multiplayer), setup::<T>)
            .add_systems(OnExit(MenuState::Multiplayer), cleanup::<T>);
    }
}

#[derive(SystemParam)]
pub(super) struct Sender<'w, T>
where
    T: LobbyRequest,
{
    counter: ResMut<'w, Counter<T>>,
    requests: EventWriter<'w, RequestEvent<T>>,
}

impl<'w, T> Sender<'w, T>
where
    T: LobbyRequest,
{
    pub(super) fn send(&mut self, request: T) {
        self.requests
            .send(RequestEvent::new(self.counter.increment(), request));
    }
}

#[derive(SystemParam)]
pub(super) struct Receiver<'w, 's, T>
where
    T: LobbyRequest,
{
    counter: Res<'w, Counter<T>>,
    responses: EventReader<'w, 's, ResponseEvent<T>>,
}

impl<'w, 's, T> Receiver<'w, 's, T>
where
    T: LobbyRequest,
{
    /// Returns the response result corresponding the ID of the last request.
    /// Responses to earlier requests or requests not made via Sender are
    /// ignored.
    pub(super) fn receive(&mut self) -> Option<&Result<T::Response>> {
        self.responses
            .iter()
            .filter_map(|e| {
                if self.counter.compare(e.id()) {
                    Some(e.result())
                } else {
                    None
                }
            })
            .last()
    }
}

#[derive(Resource)]
pub(super) struct Counter<T>
where
    T: LobbyRequest,
{
    counter: u64,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Counter<T>
where
    T: LobbyRequest,
{
    fn new(initial_value: u64) -> Self {
        Self {
            counter: initial_value,
            _marker: PhantomData,
        }
    }

    fn increment(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        self.counter
    }

    fn compare(&self, id: &str) -> bool {
        self.counter.to_string() == id
    }
}

fn setup<T>(mut commands: Commands, time: Res<Time>)
where
    T: LobbyRequest,
{
    let init = time.elapsed().as_secs().wrapping_mul(1024);
    commands.insert_resource(Counter::<T>::new(init));
}

fn cleanup<T>(mut commands: Commands)
where
    T: LobbyRequest,
{
    commands.remove_resource::<Counter<T>>();
}
