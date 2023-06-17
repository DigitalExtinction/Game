use async_std::{channel::bounded, io, task};
pub use communicator::{
    ConnErrorReceiver, InMessage, MessageReceiver, MessageSender, OutMessage, OutMessageBuilder,
};
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
pub fn startup(
    network: Network,
) -> io::Result<(MessageSender, MessageReceiver, ConnErrorReceiver)> {
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
    let (sreceiver_cancellation_sender, sreceiver_cancellation_receiver) = cancellation();
    task::spawn(sreceiver::run(
        port,
        sreceiver_cancellation_receiver,
        in_system_datagrams_receiver,
        resends.clone(),
    ));

    let (inputs_sender, inputs_receiver) = bounded(CHANNEL_CAPACITY);
    let (confirmer_cancellation_sender, confirmer_cancellation_receiver) = cancellation();
    let confirms = Confirmations::new();
    task::spawn(ureceiver::run(
        port,
        confirmer_cancellation_sender,
        in_user_datagrams_receiver,
        inputs_sender,
        confirms.clone(),
    ));

    let (outputs_sender, outputs_receiver) = bounded(CHANNEL_CAPACITY);
    let (errors_sender, errors_receiver) = bounded(CHANNEL_CAPACITY);
    let (resender_cancellation_sender, resender_cancellation_receiver) = cancellation();
    task::spawn(resender::run(
        port,
        resender_cancellation_receiver,
        sreceiver_cancellation_sender,
        out_datagrams_sender.clone(),
        errors_sender,
        resends.clone(),
    ));

    task::spawn(confirmer::run(
        port,
        confirmer_cancellation_receiver,
        out_datagrams_sender.clone(),
        confirms,
    ));
    task::spawn(usender::run(
        port,
        resender_cancellation_sender,
        out_datagrams_sender,
        outputs_receiver,
        resends,
    ));

    Ok((
        MessageSender(outputs_sender),
        MessageReceiver(inputs_receiver),
        ConnErrorReceiver(errors_receiver),
    ))
}
