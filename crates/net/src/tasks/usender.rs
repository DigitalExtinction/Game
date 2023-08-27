use std::time::Instant;

use async_std::channel::{Receiver, Sender};
use tracing::{error, info};

use super::{cancellation::CancellationSender, dsender::OutDatagram};
use crate::{
    connection::DispatchHandler,
    header::{DatagramHeader, PackageHeader, PackageIdRange},
    OutPackage,
};

/// Handler & scheduler of datagram resends.
pub(super) async fn run(
    port: u16,
    _cancellation: CancellationSender,
    datagrams: Sender<OutDatagram>,
    packages: Receiver<OutPackage>,
    mut dispatch_handler: DispatchHandler,
) {
    info!("Starting package sender on port {port}...");

    let mut counter_unreliable = PackageIdRange::counter();

    loop {
        let Ok(package) = packages.recv().await else {
            break;
        };

        let time = Instant::now();
        let target = package.target();

        let package_id = if package.reliability().is_reliable() {
            dispatch_handler.next_package_id(time, target).await
        } else {
            counter_unreliable.next().unwrap()
        };

        let package_header = PackageHeader::new(package.reliability(), package.peers(), package_id);
        let header = DatagramHeader::Package(package_header);

        if package_header.reliability().is_reliable() {
            dispatch_handler
                .sent(time, target, package_header, package.data_slice())
                .await;
        }

        let closed = datagrams
            .send(OutDatagram::new(header, package.data(), target))
            .await
            .is_err();

        if closed {
            error!("Datagram sender channel on port {port} is unexpectedly closed. ");
            break;
        }
    }

    info!("Package sender on port {port} finished.");
}
