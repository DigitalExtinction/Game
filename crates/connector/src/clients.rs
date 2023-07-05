use std::{collections::hash_map::Entry, net::SocketAddr};

use ahash::AHashMap;
use async_std::sync::{Arc, RwLock};

/// Registry of clients and the games (their ports) they registered into.
#[derive(Clone)]
pub(crate) struct Clients {
    inner: Arc<RwLock<ClientsInner>>,
}

impl Clients {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ClientsInner::new())),
        }
    }

    /// Reserves game spot for the client so they can later join the game.
    /// Returns error if the reservation was NOT successful, i.e. the client is
    /// joined to a game with the given port or has a spot reservation.
    pub(crate) async fn reserve(&mut self, addr: SocketAddr) -> Result<(), String> {
        self.inner.write().await.reserve(addr)
    }

    /// Frees the spot for the client. Call this for example after the client
    /// disconnects from a game.
    pub(crate) async fn free(&mut self, addr: SocketAddr) {
        self.inner.write().await.free(addr)
    }

    /// Sets game for a client with a reservation. See [`Self::reserve`].
    ///
    /// # Panics
    ///
    /// Panics if the `addr` is not in reserved state.
    pub(crate) async fn set(&mut self, addr: SocketAddr, game_port: u16) {
        self.inner.write().await.set(addr, game_port)
    }
}

struct ClientsInner {
    socket_to_game: AHashMap<SocketAddr, Option<u16>>,
}

impl ClientsInner {
    fn new() -> Self {
        Self {
            socket_to_game: AHashMap::new(),
        }
    }

    fn reserve(&mut self, addr: SocketAddr) -> Result<(), String> {
        match self.socket_to_game.entry(addr) {
            Entry::Vacant(entry) => {
                entry.insert(None);
                Ok(())
            }
            Entry::Occupied(entry) => match *entry.get() {
                Some(port) => Err(format!("Client {addr:?} is already in game on port {port}")),
                None => Err(format!("Client {addr:?} already has a game reservation")),
            },
        }
    }

    fn free(&mut self, addr: SocketAddr) {
        self.socket_to_game.remove(&addr);
    }

    fn set(&mut self, addr: SocketAddr, game_port: u16) {
        match self.socket_to_game.entry(addr) {
            Entry::Vacant(_) => {
                panic!("Spot not reserved for {addr:?}.");
            }
            Entry::Occupied(mut entry) => {
                if let Some(previous) = entry.insert(Some(game_port)) {
                    panic!("Client {addr:?} is already in game {previous}.");
                }
            }
        }
    }
}
