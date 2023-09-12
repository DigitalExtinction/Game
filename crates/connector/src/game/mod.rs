use std::net::SocketAddr;

use async_std::{channel::bounded, task};
use de_net::{self, Socket};
use de_types::player::Player;

use self::{greceiver::GameProcessor, state::GameState};
use crate::clients::Clients;

mod buffer;
mod ereceiver;
mod greceiver;
mod message;
mod mreceiver;
mod preceiver;
mod state;

/// Startup game network server communicating via `net`.
///
/// # Arguments
///
/// * `clients` - global clients tracker.
///
/// * `socket` - socket to use for the game server.
///
/// * `owner` - address of the creator of the game. This client will be
///   automatically added to the game as if they sent
///   [`de_messages::ToGame::Join`].
///
/// * `max_players` - maximum number of clients which may connect to the game
///   at the same time
pub(crate) async fn startup(
    clients: Clients,
    socket: Socket,
    owner: SocketAddr,
    max_players: Player,
) {
    let port = socket.port();
    let (outputs, inputs, errors) = de_net::startup(
        |t| {
            task::spawn(t);
        },
        socket,
    );

    let (server_sender, server_receiver) = bounded(16);
    task::spawn(ereceiver::run(port, errors, server_sender.clone()));

    let (players_sender, players_receiver) = bounded(16);
    task::spawn(mreceiver::run(port, inputs, server_sender, players_sender));

    let state = GameState::new(max_players);
    let server = GameProcessor::new(
        port,
        owner,
        server_receiver,
        outputs.clone(),
        state.clone(),
        clients,
    );
    task::spawn(server.run());

    task::spawn(preceiver::run(port, players_receiver, outputs, state));
}
