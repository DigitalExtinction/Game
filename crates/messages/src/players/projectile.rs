use bincode::{Decode, Encode};

use crate::players::Vec3Net;

#[derive(Clone, Copy, Debug, Encode, Decode)]
pub enum NetProjectile {
    Laser {
        origin: Vec3Net,
        /// End of the trail lies at `origin + direction`.
        direction: Vec3Net,
    },
}
