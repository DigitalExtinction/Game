use std::marker::PhantomData;

use ahash::AHashMap;
use anyhow::Result;
use bevy::prelude::*;
use bevy::tasks::Task;
use futures_lite::future;

use crate::{
    client::AuthenticatedClient,
    requestable::{LobbyRequest, LobbyRequestCreator},
};

pub(super) struct EndpointPlugin<T: LobbyRequestCreator> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: LobbyRequestCreator> Plugin for EndpointPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_event::<RequestEvent<T>>()
            .add_event::<ResponseEvent<T>>()
            .init_resource::<PendingTasks<T>>()
            .add_system(fire::<T>)
            .add_system(poll::<T>);
    }
}

impl<T: LobbyRequestCreator> Default for EndpointPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// Use this event to make a request the Lobby API. Response the request will
/// delivered as [`ResponseEvent`].
pub struct RequestEvent<T> {
    id: String,
    request: T,
}

impl<T> RequestEvent<T> {
    /// # Arguments
    ///
    /// * `id` - ID of the request. The response event to this request will
    ///   have the same ID. Any pending request with the same ID and request
    ///   type will get dropped & canceled.
    ///
    /// * `request` - the request to be made itself.
    pub fn new<S: ToString>(id: S, request: T) -> Self {
        Self {
            id: id.to_string(),
            request,
        }
    }

    fn id(&self) -> &str {
        self.id.as_str()
    }

    fn request(&self) -> &T {
        &self.request
    }
}

/// Event corresponding to a finished Lobby API request which might have failed
/// or succeeded.
pub struct ResponseEvent<T>
where
    T: LobbyRequest,
{
    id: String,
    result: Result<T::Response>,
}

impl<T> ResponseEvent<T>
where
    T: LobbyRequest,
{
    fn new(id: String, result: Result<T::Response>) -> Self {
        Self { id, result }
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn result(&self) -> &Result<T::Response> {
        &self.result
    }
}

#[derive(Resource)]
struct PendingTasks<T: LobbyRequest>(AHashMap<String, Task<Result<T::Response>>>);

impl<T: LobbyRequest> PendingTasks<T> {
    fn register(&mut self, id: String, task: Task<Result<T::Response>>) {
        self.0.insert(id, task);
    }
}

impl<T: LobbyRequest> Default for PendingTasks<T> {
    fn default() -> Self {
        Self(AHashMap::new())
    }
}

fn fire<T: LobbyRequestCreator>(
    client: AuthenticatedClient,
    mut pending: ResMut<PendingTasks<T>>,
    mut requests: EventReader<RequestEvent<T>>,
    mut responses: EventWriter<ResponseEvent<T>>,
) {
    for event in requests.iter() {
        let result = client.fire(event.request());

        match result {
            Ok(task) => pending.register(event.id().to_owned(), task),
            Err(error) => responses.send(ResponseEvent::new(event.id().to_owned(), Err(error))),
        }
    }
}

fn poll<T: LobbyRequest>(
    mut pending: ResMut<PendingTasks<T>>,
    mut events: EventWriter<ResponseEvent<T>>,
) {
    let mut results = Vec::new();

    for (id, task) in pending.0.iter_mut() {
        if task.is_finished() {
            match future::block_on(future::poll_once(task)) {
                Some(result) => results.push((id.to_owned(), result)),
                None => unreachable!("The task is finished."),
            }
        }
    }

    for result in results.drain(..) {
        pending.0.remove(result.0.as_str());
        events.send(ResponseEvent::new(result.0, result.1));
    }
}
