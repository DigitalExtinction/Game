use async_std::{channel::bounded, io, task};
pub use communicator::{Communicator, InMessage, OutMessage, OutMessageBuilder};
pub(crate) use dsender::OutDatagram;
use tracing::info;

use crate::{
    connection::{Confirmations, Resends},
    messages::Messages,
    tasks::cancellation::cancellation,
    Network,
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

/// Setups and starts communication stack tasks.
pub fn startup(network: Network) -> io::Result<Communicator> {
    let port = network.port()?;
    info!("Starting up network stack on port {port}...");

    let messages = Messages::new(network);

    let (out_datagrams_sender, out_datagrams_receiver) = bounded(16);
    task::spawn(dsender::run(port, out_datagrams_receiver, messages.clone()));

    let (in_system_datagrams_sender, in_system_datagrams_receiver) = bounded(16);
    let (in_user_datagrams_sender, in_user_datagrams_receiver) = bounded(16);
    task::spawn(dreceiver::run(
        port,
        in_system_datagrams_sender,
        in_user_datagrams_sender,
        messages,
    ));

    let resends = Resends::new();
    task::spawn(sreceiver::run(
        port,
        in_system_datagrams_receiver,
        resends.clone(),
    ));

    let (inputs_sender, inputs_receiver) = bounded(CHANNEL_CAPACITY);
    let (cancellation_sender, cancellation_receiver) = cancellation();
    let confirms = Confirmations::new();
    task::spawn(ureceiver::run(
        port,
        cancellation_sender,
        in_user_datagrams_receiver,
        inputs_sender,
        confirms.clone(),
    ));

    let (outputs_sender, outputs_receiver) = bounded(CHANNEL_CAPACITY);
    let (errors_sender, errors_receiver) = bounded(CHANNEL_CAPACITY);
    task::spawn(resender::run(
        port,
        out_datagrams_sender.clone(),
        errors_sender,
        resends.clone(),
    ));

    task::spawn(confirmer::run(
        port,
        cancellation_receiver,
        out_datagrams_sender.clone(),
        confirms,
    ));
    task::spawn(usender::run(
        port,
        out_datagrams_sender,
        outputs_receiver,
        resends,
    ));

    Ok(Communicator::new(
        outputs_sender,
        inputs_receiver,
        errors_receiver,
    ))
}
