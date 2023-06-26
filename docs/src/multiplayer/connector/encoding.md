# Message Encoding

Individual messages, implemented as Rust enums, are encoded via
[Bincode](https://github.com/bincode-org/bincode) version 2. Usually, multiple
messages are sent in a single datagram. Big endian with variable bit encoding
is used.

See individual message [documentation](/rust/de_net/protocol/).
