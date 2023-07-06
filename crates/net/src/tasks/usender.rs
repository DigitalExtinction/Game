use std::time::Instant;

use async_std::channel::{Receiver, Sender};
use tracing::{error, info};

use super::{cancellation::CancellationSender, dsender::OutDatagram};
use crate::{
    connection::Resends,
    header::{DatagramHeader, PackageId},
    OutPackage,
};

/// Handler & scheduler of datagram resends.
pub(super) async fn run(
    port: u16,
    _cancellation: CancellationSender,
    datagrams: Sender<OutDatagram>,
    packages: Receiver<OutPackage>,
    mut resends: Resends,
) {
    info!("Starting package sender on port {port}...");

    let mut counter_reliable = PackageId::zero();
    let mut counter_unreliable = PackageId::zero();

    loop {
        let Ok(package) = packages.recv().await else {
            break;
        };

        let counter = if package.reliable() {
            &mut counter_reliable
        } else {
            &mut counter_unreliable
        };

        let header = DatagramHeader::new_package(package.reliable(), package.peers(), *counter);
        *counter = counter.incremented();

        if let DatagramHeader::Package(package_header) = header {
            if package_header.reliable() {
                let time = Instant::now();
                for target in &package.targets {
                    resends
                        .sent(
                            time,
                            target,
                            package_header.id(),
                            package_header.peers(),
                            &package.data,
                        )
                        .await;
                }
            }
        }

        let closed = datagrams
            .send(OutDatagram::new(header, package.data, package.targets))
            .await
            .is_err();

        if closed {
            error!("Datagram sender channel on port {port} is unexpectedly closed. ");
            break;
        }
    }

    info!("Package sender on port {port} finished.");
}
