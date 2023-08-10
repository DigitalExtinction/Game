use std::{collections::hash_map::Entry, net::SocketAddr};

use ahash::AHashMap;
use async_std::sync::{Arc, RwLock};
use de_net::Targets;
use thiserror::Error;

#[derive(Clone)]
pub(super) struct GameState {
    inner: Arc<RwLock<GameStateInner>>,
}

impl GameState {
    pub(super) fn new(max_players: u8) -> Self {
        Self {
            inner: Arc::new(RwLock::new(GameStateInner::new(max_players))),
        }
    }

    /// Returns true if there is no players currently connected to the game.
    pub(super) async fn is_empty(&self) -> bool {
        self.inner.read().await.is_empty()
    }

    /// Returns true if a player with `addr` is connected to the game.
    pub(super) async fn contains(&self, addr: SocketAddr) -> bool {
        self.inner.read().await.contains(addr)
    }

    /// Adds a player to the game and returns ID of the added player.
    pub(super) async fn add(&mut self, addr: SocketAddr) -> Result<u8, JoinError> {
        self.inner.write().await.add(addr)
    }

    /// Removes a single player from the game. It returns ID of the player if
    /// the player was part of the game or None otherwise.
    pub(super) async fn remove(&mut self, addr: SocketAddr) -> Option<u8> {
        self.inner.write().await.remove(addr)
    }

    /// If the game is in state `Open`, changes its state to `Starting` and
    /// returns true.
    pub(super) async fn start(&mut self) -> bool {
        self.inner.write().await.start()
    }

    /// Marks a player as initialized. Returns true if the game was just
    /// started.
    pub(super) async fn mark_initialized(&mut self, addr: SocketAddr) -> bool {
        self.inner.write().await.mark_initialized(addr)
    }

    /// Constructs and returns package targets which includes all or all but
    /// one players connected to the game. It returns None if there is no
    /// matching target.
    ///
    /// # Arguments
    ///
    /// * `exclude` - if not None, this player is included among the targets.
    pub(super) async fn targets(&self, exclude: Option<SocketAddr>) -> Option<Targets<'static>> {
        self.inner.read().await.targets(exclude)
    }
}

struct GameStateInner {
    available_ids: AvailableIds,
    state: GameStateX,
    players: AHashMap<SocketAddr, Player>,
}

impl GameStateInner {
    fn new(max_players: u8) -> Self {
        Self {
            available_ids: AvailableIds::new(max_players),
            state: GameStateX::Open,
            players: AHashMap::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    fn contains(&self, addr: SocketAddr) -> bool {
        self.players.contains_key(&addr)
    }

    fn add(&mut self, addr: SocketAddr) -> Result<u8, JoinError> {
        if self.state != GameStateX::Open {
            return Err(JoinError::GameNotOpened);
        }

        match self.players.entry(addr) {
            Entry::Occupied(_) => Err(JoinError::AlreadyJoined),
            Entry::Vacant(vacant) => match self.available_ids.lease() {
                Some(id) => {
                    vacant.insert(Player::new(id));
                    Ok(id)
                }
                None => Err(JoinError::GameFull),
            },
        }
    }

    fn remove(&mut self, addr: SocketAddr) -> Option<u8> {
        match self.players.remove_entry(&addr) {
            Some((_, player)) => {
                self.available_ids.release(player.id);
                Some(player.id)
            }
            None => None,
        }
    }

    fn start(&mut self) -> bool {
        if self.state == GameStateX::Open {
            self.state = GameStateX::Starting;
            true
        } else {
            false
        }
    }

    fn mark_initialized(&mut self, addr: SocketAddr) -> bool {
        let prev = self.state;

        if matches!(self.state, GameStateX::Starting) {
            if let Some(player) = self.players.get_mut(&addr) {
                player.initialized = true;
            }
            if self.players.values().all(|p| p.initialized) {
                self.state = GameStateX::Started;
            }
        }

        self.state == GameStateX::Started && self.state != prev
    }

    fn targets(&self, exclude: Option<SocketAddr>) -> Option<Targets<'static>> {
        let len = if exclude.map_or(false, |e| self.players.contains_key(&e)) {
            self.players.len() - 1
        } else {
            self.players.len()
        };

        if len == 0 {
            None
        } else if len == 1 {
            for &addr in self.players.keys() {
                if Some(addr) != exclude {
                    return Some(Targets::Single(addr));
                }
            }

            unreachable!("No non-excluded player found.");
        } else {
            let mut addrs = Vec::with_capacity(len);
            for &addr in self.players.keys() {
                if Some(addr) != exclude {
                    addrs.push(addr);
                }
            }
            Some(addrs.into())
        }
    }
}

struct AvailableIds(Vec<u8>);

impl AvailableIds {
    fn new(max_players: u8) -> Self {
        Self(Vec::from_iter((1..=max_players).rev()))
    }

    /// Borrows a new ID or returns None if all are already borrowed.
    fn lease(&mut self) -> Option<u8> {
        self.0.pop()
    }

    /// Makes a borrowed ID available for another borrow.
    ///
    /// # Panics
    ///
    /// Panics if the ID is not borrowed.
    fn release(&mut self, id: u8) {
        let index = match self.0.iter().position(|other| *other <= id) {
            Some(index) => {
                assert_ne!(self.0[index], id);
                index
            }
            None => self.0.len(),
        };

        self.0.insert(index, id);
    }
}

#[derive(Debug, Error, PartialEq)]
pub(super) enum JoinError {
    #[error("The player has already joined the game.")]
    AlreadyJoined,
    #[error("The game is full.")]
    GameFull,
    #[error("The game is no longer opened.")]
    GameNotOpened,
}

struct Player {
    id: u8,
    initialized: bool,
}

impl Player {
    fn new(id: u8) -> Self {
        Self {
            id,
            initialized: false,
        }
    }
}

// TODO better name
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GameStateX {
    Open,
    Starting,
    Started,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use async_std::task;

    use super::*;

    #[test]
    fn test_state() {
        task::block_on(task::spawn(async {
            let mut state = GameState::new(8);
            let mut ids: HashSet<u8> = HashSet::new();

            assert!(ids.insert(state.add("127.0.0.1:1001".parse().unwrap()).await.unwrap()));
            assert!(state.contains("127.0.0.1:1001".parse().unwrap()).await);

            assert!(ids.insert(state.add("127.0.0.1:1002".parse().unwrap()).await.unwrap()));
            assert!(state.contains("127.0.0.1:1001".parse().unwrap()).await);
            assert!(state.contains("127.0.0.1:1002".parse().unwrap()).await);

            assert!(ids.remove(
                &state
                    .remove("127.0.0.1:1001".parse().unwrap())
                    .await
                    .unwrap()
            ));
            assert!(!state.contains("127.0.0.1:1001".parse().unwrap()).await);
            assert!(state.contains("127.0.0.1:1002".parse().unwrap()).await);

            assert!(ids.insert(state.add("127.0.0.1:1001".parse().unwrap()).await.unwrap()));
            assert!(state.contains("127.0.0.1:1001".parse().unwrap()).await);
            assert!(state.contains("127.0.0.1:1002".parse().unwrap()).await);

            assert!(matches!(
                state.add("127.0.0.1:1001".parse().unwrap()).await,
                Err(JoinError::AlreadyJoined),
            ));

            for i in 3..=8 {
                assert!(ids.insert(
                    state
                        .add(format!("127.0.0.1:100{i}").parse().unwrap())
                        .await
                        .unwrap()
                ));
            }

            assert!(matches!(
                state.add("127.0.0.1:1020".parse().unwrap()).await,
                Err(JoinError::GameFull),
            ));
            assert!(!state.contains("127.0.0.1:1020".parse().unwrap()).await);
        }));
    }

    #[test]
    fn test_transitions() {
        let client_a: SocketAddr = "127.0.0.1:8081".parse().unwrap();
        let client_b: SocketAddr = "127.0.0.1:8082".parse().unwrap();
        let client_c: SocketAddr = "127.0.0.1:8083".parse().unwrap();

        let mut state = GameStateInner::new(3);

        state.add(client_a).unwrap();
        state.add(client_b).unwrap();

        assert!(state.start());
        assert!(!state.start());

        assert_eq!(state.add(client_c), Err(JoinError::GameNotOpened));

        assert!(!state.mark_initialized(client_b));
        assert!(state.mark_initialized(client_a));
    }

    #[test]
    fn test_targets() {
        let mut state = GameStateInner::new(8);

        assert!(state.targets(None).is_none());

        state.add("127.0.0.1:2001".parse().unwrap()).unwrap();
        assert_eq!(
            HashSet::<SocketAddr>::from_iter(state.targets(None).unwrap().into_iter()),
            HashSet::from_iter(["127.0.0.1:2001".parse().unwrap()])
        );
        assert!(state
            .targets(Some("127.0.0.1:2001".parse().unwrap()))
            .is_none());

        state.add("127.0.0.1:2002".parse().unwrap()).unwrap();
        state.add("127.0.0.1:2003".parse().unwrap()).unwrap();
        assert_eq!(
            HashSet::<SocketAddr>::from_iter(state.targets(None).unwrap().into_iter()),
            HashSet::from_iter([
                "127.0.0.1:2001".parse().unwrap(),
                "127.0.0.1:2002".parse().unwrap(),
                "127.0.0.1:2003".parse().unwrap()
            ])
        );
        assert_eq!(
            HashSet::<SocketAddr>::from_iter(
                state
                    .targets(Some("127.0.0.1:2002".parse().unwrap()))
                    .unwrap()
                    .into_iter()
            ),
            HashSet::from_iter([
                "127.0.0.1:2001".parse().unwrap(),
                "127.0.0.1:2003".parse().unwrap(),
            ])
        );
    }

    #[test]
    fn test_available_ids() {
        let mut ids = AvailableIds::new(3);

        assert_eq!(ids.lease().unwrap(), 1);
        assert_eq!(ids.lease().unwrap(), 2);
        assert_eq!(ids.lease().unwrap(), 3);
        assert!(ids.lease().is_none());

        ids.release(2);
        ids.release(3);
        ids.release(1);
        assert_eq!(ids.lease().unwrap(), 1);
        assert_eq!(ids.lease().unwrap(), 2);
        assert_eq!(ids.lease().unwrap(), 3);
        assert!(ids.lease().is_none());
    }
}
