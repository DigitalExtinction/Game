use std::{collections::hash_map::Entry, net::SocketAddr};

use ahash::AHashMap;
use async_std::sync::{Arc, RwLock, RwLockWriteGuard};
use de_messages::Readiness;
use de_types::player::{Player, PlayerRange};
use thiserror::Error;

use super::buffer::PlayerBuffer;

#[derive(Clone)]
pub(super) struct GameState {
    inner: Arc<RwLock<GameStateInner>>,
}

impl GameState {
    pub(super) fn new(max_players: Player) -> Self {
        Self {
            inner: Arc::new(RwLock::new(GameStateInner::new(max_players))),
        }
    }

    pub(crate) async fn lock(&mut self) -> GameStateGuard {
        GameStateGuard {
            guard: self.inner.write().await,
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

    /// Returns ID of the player or None if such player is not part of the
    /// game.
    pub(super) async fn id(&self, addr: SocketAddr) -> Option<Player> {
        self.inner.read().await.id(addr)
    }

    /// Adds a player to the game and returns ID of the added player.
    pub(super) async fn add(&mut self, addr: SocketAddr) -> Result<Player, JoinError> {
        self.inner.write().await.add(addr)
    }

    /// Removes a single player from the game. It returns ID of the player if
    /// the player was part of the game or None otherwise.
    pub(super) async fn remove(&mut self, addr: SocketAddr) -> Option<Player> {
        self.inner.write().await.remove(addr)
    }

    /// Updates readiness of a single player. Whole game readiness is updated
    /// once all players reach another readiness stage.
    ///
    /// Returns true if game readiness progressed as a result (to the readiness
    /// of the player).
    pub(super) async fn update_readiness(
        &mut self,
        addr: SocketAddr,
        readiness: Readiness,
    ) -> Result<bool, ReadinessUpdateError> {
        self.inner.write().await.update_readiness(addr, readiness)
    }

    /// Constructs and returns package targets which includes all or all but
    /// one players connected to the game.
    ///
    /// # Arguments
    ///
    /// * `exclude` - if not None, this player is included among the targets.
    pub(super) async fn targets(&self, exclude: Option<SocketAddr>) -> Vec<SocketAddr> {
        self.inner.read().await.targets(exclude)
    }
}

/// The lock is unlocked once this guard is dropped.
pub(super) struct GameStateGuard<'a> {
    guard: RwLockWriteGuard<'a, GameStateInner>,
}

impl<'a> GameStateGuard<'a> {
    /// Returns an iterator over message buffers of all or all but one player.
    ///
    /// # Arguments
    ///
    /// * `exclude` - exclude this player from the iterator.
    pub(super) fn buffers_mut(
        &mut self,
        exclude: Option<SocketAddr>,
    ) -> impl Iterator<Item = &mut PlayerBuffer> {
        self.guard.buffers_mut(exclude)
    }
}

struct GameStateInner {
    available_ids: AvailableIds,
    readiness: Readiness,
    players: AHashMap<SocketAddr, PlayerSlot>,
}

impl GameStateInner {
    fn new(max_players: Player) -> Self {
        Self {
            available_ids: AvailableIds::new(max_players),
            readiness: Readiness::default(),
            players: AHashMap::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    fn contains(&self, addr: SocketAddr) -> bool {
        self.players.contains_key(&addr)
    }

    fn id(&self, addr: SocketAddr) -> Option<Player> {
        self.players.get(&addr).map(|p| p.id)
    }

    fn add(&mut self, addr: SocketAddr) -> Result<Player, JoinError> {
        if self.readiness != Readiness::NotReady {
            return Err(JoinError::GameNotOpened);
        }

        match self.players.entry(addr) {
            Entry::Occupied(_) => Err(JoinError::AlreadyJoined),
            Entry::Vacant(vacant) => match self.available_ids.lease() {
                Some(id) => {
                    vacant.insert(PlayerSlot::new(id, addr));
                    Ok(id)
                }
                None => Err(JoinError::GameFull),
            },
        }
    }

    fn remove(&mut self, addr: SocketAddr) -> Option<Player> {
        match self.players.remove_entry(&addr) {
            Some((_, player)) => {
                self.available_ids.release(player.id);
                Some(player.id)
            }
            None => None,
        }
    }

    fn update_readiness(
        &mut self,
        addr: SocketAddr,
        readiness: Readiness,
    ) -> Result<bool, ReadinessUpdateError> {
        let Some(player) = self.players.get_mut(&addr) else {
            return Err(ReadinessUpdateError::UnknownClient(addr));
        };

        if player.readiness > readiness {
            return Err(ReadinessUpdateError::Downgrade {
                from: player.readiness,
                to: readiness,
            });
        }

        if player.readiness == readiness {
            return Ok(false);
        }

        if player.readiness.progress() != Some(readiness) {
            return Err(ReadinessUpdateError::Skip {
                from: player.readiness,
                to: readiness,
            });
        }

        if player.readiness > self.readiness {
            // The player is already ahead of the game, cannot move them further.
            return Err(ReadinessUpdateError::Desync {
                game: self.readiness,
                client: readiness,
            });
        }

        player.readiness = readiness;

        let previous = self.readiness;
        self.readiness = self.players.values().map(|p| p.readiness).min().unwrap();
        let progressed = previous != self.readiness;
        assert!(self.readiness == readiness || !progressed);
        Ok(progressed)
    }

    fn targets(&self, exclude: Option<SocketAddr>) -> Vec<SocketAddr> {
        let mut addrs = Vec::with_capacity(self.players.len());
        for &addr in self.players.keys() {
            if Some(addr) != exclude {
                addrs.push(addr);
            }
        }
        addrs
    }

    fn buffers_mut(
        &mut self,
        exclude: Option<SocketAddr>,
    ) -> impl Iterator<Item = &mut PlayerBuffer> {
        self.players.iter_mut().filter_map(move |(&addr, player)| {
            if Some(addr) == exclude {
                None
            } else {
                Some(&mut player.buffer)
            }
        })
    }
}

struct AvailableIds(Vec<Player>);

impl AvailableIds {
    fn new(max_players: Player) -> Self {
        let mut ids = Vec::from_iter(PlayerRange::up_to(max_players));
        ids.reverse();
        Self(ids)
    }

    /// Borrows a new ID or returns None if all are already borrowed.
    fn lease(&mut self) -> Option<Player> {
        self.0.pop()
    }

    /// Makes a borrowed ID available for another borrow.
    ///
    /// # Panics
    ///
    /// Panics if the ID is not borrowed.
    fn release(&mut self, id: Player) {
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

#[derive(Debug, Error, PartialEq)]
pub(super) enum ReadinessUpdateError {
    #[error("Client {0:?} is not part of the game.")]
    UnknownClient(SocketAddr),
    #[error("Cannot downgrade client readiness from {from:?} to {to:?}.")]
    Downgrade { from: Readiness, to: Readiness },
    #[error("Cannot upgrade client readiness from {from:?} to {to:?}.")]
    Skip { from: Readiness, to: Readiness },
    #[error("Cannot change client readiness to {client:?} when game is at {game:?}.")]
    Desync { game: Readiness, client: Readiness },
}

struct PlayerSlot {
    id: Player,
    readiness: Readiness,
    buffer: PlayerBuffer,
}

impl PlayerSlot {
    fn new(id: Player, addr: SocketAddr) -> Self {
        Self {
            id,
            readiness: Readiness::default(),
            buffer: PlayerBuffer::new(addr),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use async_std::task;

    use super::*;

    #[test]
    fn test_state() {
        task::block_on(task::spawn(async {
            let mut state = GameState::new(Player::Player4);
            let mut ids: HashSet<Player> = HashSet::new();

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

            for i in 3..=4 {
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

        let mut state = GameStateInner::new(Player::Player3);

        state.add(client_a).unwrap();
        state.add(client_b).unwrap();

        assert_eq!(state.readiness, Readiness::NotReady);

        assert!(!state
            .update_readiness(client_a, Readiness::NotReady)
            .unwrap());
        assert_eq!(state.readiness, Readiness::NotReady);

        assert!(!state.update_readiness(client_b, Readiness::Ready).unwrap());
        assert_eq!(state.readiness, Readiness::NotReady);
        assert!(state.update_readiness(client_a, Readiness::Ready).unwrap());
        assert_eq!(state.readiness, Readiness::Ready);

        assert_eq!(state.add(client_c), Err(JoinError::GameNotOpened));

        assert_eq!(
            state
                .update_readiness(client_a, Readiness::Initialized)
                .unwrap_err(),
            ReadinessUpdateError::Skip {
                from: Readiness::Ready,
                to: Readiness::Initialized
            }
        );

        assert!(!state
            .update_readiness(client_b, Readiness::Prepared)
            .unwrap());
        assert_eq!(
            state
                .update_readiness(client_b, Readiness::Initialized)
                .unwrap_err(),
            ReadinessUpdateError::Desync {
                game: Readiness::Ready,
                client: Readiness::Initialized
            }
        );
        assert_eq!(state.readiness, Readiness::Ready);

        assert!(state
            .update_readiness(client_a, Readiness::Prepared)
            .unwrap());
        assert_eq!(state.readiness, Readiness::Prepared);

        assert!(!state
            .update_readiness(client_a, Readiness::Initialized)
            .unwrap());
        assert_eq!(state.readiness, Readiness::Prepared);
        assert!(state
            .update_readiness(client_b, Readiness::Initialized)
            .unwrap());
        assert_eq!(state.readiness, Readiness::Initialized);
    }

    #[test]
    fn test_targets() {
        let mut state = GameStateInner::new(Player::Player4);

        assert!(state.targets(None).is_empty());

        state.add("127.0.0.1:2001".parse().unwrap()).unwrap();
        assert_eq!(
            HashSet::<SocketAddr>::from_iter(state.targets(None).into_iter()),
            HashSet::from_iter(["127.0.0.1:2001".parse().unwrap()])
        );
        assert!(state
            .targets(Some("127.0.0.1:2001".parse().unwrap()))
            .is_empty());

        state.add("127.0.0.1:2002".parse().unwrap()).unwrap();
        state.add("127.0.0.1:2003".parse().unwrap()).unwrap();
        assert_eq!(
            HashSet::<SocketAddr>::from_iter(state.targets(None).into_iter()),
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
        let mut ids = AvailableIds::new(Player::Player3);

        assert_eq!(ids.lease().unwrap(), Player::Player1);
        assert_eq!(ids.lease().unwrap(), Player::Player2);
        assert_eq!(ids.lease().unwrap(), Player::Player3);
        assert!(ids.lease().is_none());

        ids.release(Player::Player2);
        ids.release(Player::Player3);
        ids.release(Player::Player1);
        assert_eq!(ids.lease().unwrap(), Player::Player1);
        assert_eq!(ids.lease().unwrap(), Player::Player2);
        assert_eq!(ids.lease().unwrap(), Player::Player3);
        assert!(ids.lease().is_none());
    }
}
