use bincode::config::{BigEndian, Configuration, Limit, Varint};
pub use builder::{PackageBuilder, PackageIterator};
pub use channels::{ConnErrorReceiver, ConnectionError, PackageReceiver, PackageSender};
pub use decode::{InPackage, MessageDecoder};
pub use encode::OutPackage;

use crate::protocol::MAX_PACKAGE_SIZE;

mod builder;
mod channels;
mod decode;
mod encode;

const BINCODE_CONF: Configuration<BigEndian, Varint, Limit<MAX_PACKAGE_SIZE>> =
    bincode::config::standard()
        .with_big_endian()
        .with_variable_int_encoding()
        .with_limit::<MAX_PACKAGE_SIZE>();
