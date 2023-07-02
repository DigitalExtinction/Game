use std::net::SocketAddr;

use async_std::{channel::bounded, task};
use de_net::{self, Socket};

use self::{greceiver::GameProcessor, state::GameState};

mod ereceiver;
mod greceiver;
mod mreceiver;
mod preceiver;
mod state;

/// Startup game network server communicating via `net`.
///
/// # Arguments
///
/// * `net` - network interface to use for the game server.
///
/// * `owner` - address of the creator of the game. This client will be
///   automatically added to the game as if they sent [`de_net::ToGame::Join`].
pub(crate) async fn startup(net: Socket, owner: SocketAddr) {
    let port = net.port();
    let (outputs, inputs, errors) = de_net::startup(
        |t| {
            task::spawn(t);
        },
        net,
    );

    let (server_sender, server_receiver) = bounded(16);
    task::spawn(ereceiver::run(port, errors, server_sender.clone()));

    let (players_sender, players_receiver) = bounded(16);
    task::spawn(mreceiver::run(port, inputs, server_sender, players_sender));

    let state = GameState::new();
    let server = GameProcessor::new(port, owner, server_receiver, outputs.clone(), state.clone());
    task::spawn(server.run());

    task::spawn(preceiver::run(port, players_receiver, outputs, state));
}
