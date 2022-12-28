use std::marker::PhantomData;

use ahash::AHashMap;
use anyhow::Result;
use bevy::prelude::*;
use bevy::tasks::Task;
use futures_lite::future;

use crate::{client::LobbyClient, requestable::Requestable};

pub(super) struct EndpointPlugin<T: Requestable> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: Requestable> Plugin for EndpointPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_event::<RequestEvent<T>>()
            .add_event::<ResponseEvent<T::Response>>()
            .init_resource::<PendingTasks<T>>()
            .add_system(fire::<T>)
            .add_system(poll::<T>);
    }
}

impl<T: Requestable> Default for EndpointPlugin<T> {
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
pub struct ResponseEvent<R> {
    id: String,
    result: Result<R>,
}

impl<R> ResponseEvent<R> {
    fn new(id: String, result: Result<R>) -> Self {
        Self { id, result }
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn result(&self) -> &Result<R> {
        &self.result
    }
}

#[derive(Resource)]
struct PendingTasks<T: Requestable>(AHashMap<String, Task<Result<T::Response>>>);

impl<T: Requestable> PendingTasks<T> {
    fn register(&mut self, id: String, task: Task<Result<T::Response>>) {
        self.0.insert(id, task);
    }
}

impl<T: Requestable> Default for PendingTasks<T> {
    fn default() -> Self {
        Self(AHashMap::new())
    }
}

fn fire<T: Requestable>(
    client: Option<Res<LobbyClient>>,
    mut pending: ResMut<PendingTasks<T>>,
    mut requests: EventReader<RequestEvent<T>>,
    mut responses: EventWriter<ResponseEvent<T::Response>>,
) {
    for event in requests.iter() {
        let result = client
            .as_ref()
            .expect("A request made before the client is setup.")
            .make(event.request());

        match result {
            Ok(task) => pending.register(event.id().to_owned(), task),
            Err(error) => responses.send(ResponseEvent::new(event.id().to_owned(), Err(error))),
        }
    }
}

fn poll<T: Requestable>(
    mut pending: ResMut<PendingTasks<T>>,
    mut events: EventWriter<ResponseEvent<T::Response>>,
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
