# Network Protocol

The protocol is based on UDP datagrams and provides means of reliable and
non-reliable binary data delivery.

There are two types of datagrams:

* package – a datagram with a header and user data payload,
* protocol control – a datagram with a header and system data.

All integers are big-endian encoded.

The structure of each datagram is this:

1. flags – 1 byte
1. ID – 3 bytes
1. payload – 504 bytes or less

Protocol control is signaled by the highest bit of the flags byte (represented
by the mask `0b1000_0000`).

## Package

Each user package has an ID, encoded within the last three bytes of the
datagram header. These IDs increment until they reach the maximum value that
can be encoded within three bytes, after which the counter resets to 0. The ID
sequence for reliable and unreliable packages are independent. Each connection
/ client has independent reliable package numbering.

Packages can be transmitted in either reliable or non-reliable mode.
Reliability is signaled by the second highest bit of the flags byte
(represented by the mask `0b0100_0000`). Packages sent in reliable mode always
receive confirmation from the receiving party and are retransmitted multiple
times, with the time delay exponentially increasing until a confirmation is
obtained. Reliably sent packages are automatically deduplicated.

Packages can be targeted to the server. This is signaled by the third highest
bit of the flags byte (represented by the mask `0b0010_0000`). All other
packages are targeted to all other players who joined the same game.

Package payload comprises the user data intended for delivery.

## Protocol Control

Currently, the only type of control datagram is the delivery confirmation
datagram. All bits in the header of these datagrams, except for the first one,
are set to 0. The payload consists of IDs for all user data datagrams that have
been sent reliably and delivered successfully. Each ID is encoded using 3
bytes.
