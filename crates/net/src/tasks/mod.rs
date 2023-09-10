//! This module implements full networking stack. The stack is implemented as a
//! set of asynchronous tasks communicating via channels.
//!
//! Bellow, `--->` lines represent channels and `* * >` represent task
//! cancellation tokens.
//!
//! ```text
//!                         +-------------+
//!                         |             |
//!           +-----------+ |   usender   | * * *
//!           |             |             |     *
//!           |             +-------------+     *
//!           v                                 *
//! +-------------+         +-------------+     *
//! |             |         |             | < * *
//! |   dsender   | <-----+ |   resender  |
//! |             |         |             | * * * *
//! +-------------+         +-------------+       *
//!           ^                                   *
//!           |             +-------------+       *
//!           |             |             |       *
//!           +-----------+ |  confirmer  | < *   *
//!                         |             |   *   *
//!                         +-------------+   *   *
//!                                           *   *
//! +-------------+         +-------------+   *   *
//! |             |         |             |   *   *
//! |  dreceiver  | +-----> |  ureceiver  | * *   *
//! |             |         |             |       *
//! +-------------+         +-------------+       *
//!           +                                   *
//!           |             +-------------+       *
//!           |             |             |       *
//!           +-----------> |  sreceiver  | < * * *
//!                         |             |
//!                         +-------------+
//! ```
//!
//! `dsender` and `dreceiver` are responsible for sending and receiving UDP
//! datagrams. Both are terminated soon after all their channels are closed.
//!
//! `resender` is responsible for redelivery of reliably sent datagrams whose
//! confirmation was not received within a time limit. If all attempts fail,
//! the user is informed via [`ConnErrorReceiver`].
//!
//! `sreceiver` is responsible for processing of system / protocol datagrams.
//! These include delivery confirmations.
//!
//! `confirmer` is responsible for sending of datagram delivery confirmations.
//!
//! `resender`, `sreceiver`, and `confirmer` are terminated soon after their
//! cancellation token is canceled.
//!
//! `usender` and `ureceiver` are responsible for sending and reception of user
//! data. The user communicates with these via [`PackageSender`] and
//! [`PackageReceiver`] respectively.

use async_std::channel::bounded;
pub use communicator::{
    ConnErrorReceiver, ConnectionError, InPackage, MessageDecoder, OutPackage, PackageBuilder,
    PackageIterator, PackageReceiver, PackageSender,
};
pub(crate) use dsender::OutDatagram;
use futures::future::BoxFuture;
use tracing::info;

use crate::{
    connection::{DeliveryHandler, DispatchHandler},
    protocol::ProtocolSocket,
    tasks::cancellation::cancellation,
    Socket,
};

mod cancellation;
mod communicator;
mod confirmer;
mod dreceiver;
mod dsender;
mod resender;
mod sreceiver;
mod ureceiver;
mod usender;

const CHANNEL_CAPACITY: usize = 1024;

/// Setups and starts communication stack tasks and returns communication
/// channels for data sending, data retrieval, and error retrieval.
///
/// All tasks in the network stack keep running until the returned channels are
/// closed. Once the [`PackageSender`], [`PackageReceiver`], and
/// [`ConnErrorReceiver`] are all dropped, the networking stack will terminate
/// completely.
///
/// # Arguments
///
/// * `spawn` - async task spawner.
///
/// * `socket` - network communication will happen over this socket.
pub fn startup<S>(spawn: S, socket: Socket) -> (PackageSender, PackageReceiver, ConnErrorReceiver)
where
    S: Fn(BoxFuture<'static, ()>),
{
    let port = socket.port();
    info!("Starting up network stack on port {port}...");

    let protocol_socket = ProtocolSocket::new(socket);

    let (out_datagrams_sender, out_datagrams_receiver) = bounded(16);
    spawn(Box::pin(dsender::run(
        port,
        out_datagrams_receiver,
        protocol_socket.clone(),
    )));

    let (in_system_datagrams_sender, in_system_datagrams_receiver) = bounded(16);
    let (in_user_datagrams_sender, in_user_datagrams_receiver) = bounded(16);
    spawn(Box::pin(dreceiver::run(
        port,
        in_system_datagrams_sender,
        in_user_datagrams_sender,
        protocol_socket,
    )));

    let dispatch_handler = DispatchHandler::new();
    let (sreceiver_cancellation_sender, sreceiver_cancellation_receiver) = cancellation();
    spawn(Box::pin(sreceiver::run(
        port,
        sreceiver_cancellation_receiver,
        in_system_datagrams_receiver,
        dispatch_handler.clone(),
    )));

    let (inputs_sender, inputs_receiver) = bounded(CHANNEL_CAPACITY);
    let (confirmer_cancellation_sender, confirmer_cancellation_receiver) = cancellation();
    let delivery_handler = DeliveryHandler::new();
    spawn(Box::pin(ureceiver::run(
        port,
        confirmer_cancellation_sender,
        in_user_datagrams_receiver,
        inputs_sender,
        delivery_handler.clone(),
    )));

    let (outputs_sender, outputs_receiver) = bounded(CHANNEL_CAPACITY);
    let (errors_sender, errors_receiver) = bounded(CHANNEL_CAPACITY);
    let (resender_cancellation_sender, resender_cancellation_receiver) = cancellation();
    spawn(Box::pin(resender::run(
        port,
        resender_cancellation_receiver,
        sreceiver_cancellation_sender,
        out_datagrams_sender.clone(),
        errors_sender,
        dispatch_handler.clone(),
    )));

    spawn(Box::pin(confirmer::run(
        port,
        confirmer_cancellation_receiver,
        out_datagrams_sender.clone(),
        delivery_handler,
    )));
    spawn(Box::pin(usender::run(
        port,
        resender_cancellation_sender,
        out_datagrams_sender,
        outputs_receiver,
        dispatch_handler,
    )));

    (
        PackageSender(outputs_sender),
        PackageReceiver(inputs_receiver),
        ConnErrorReceiver(errors_receiver),
    )
}
