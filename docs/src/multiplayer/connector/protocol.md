# Network Protocol

The protocol is based on UDP datagrams. It provides means of reliable and
non-reliable binary data delivery.

All integers are big-endian encoded.

User data can be delivered either reliably or non-reliably. User data can be
targeted either to the server or to other players who joined the same game.

There are two types of datagrams:

* protocol control datagrams,
* user data datagrams.

The structure of each datagram is this:

1. datagram flags – 1 bytes
1. datagram ID – 3 bytes
1. datagram payload – 504 bytes or less

Protocol control is signaled by the highest bit of the flags byte (represented
by the mask `0b1000_0000`).

## User Data

Each user data datagram possesses an ID, encoded within the last three bytes of
the datagram header. These IDs increment until they reach the maximum value
that can be encoded within three bytes, after which the counter resets to 0.

User data can be transmitted in either reliable or non-reliable mode.
Reliability is signaled by the second highest bit of the flags byte
(represented by the mask `0b0100_0000`). Data sent in reliable mode always
receives confirmation from the receiving party and is retransmitted multiple
times, with the time delay exponentially increasing until a confirmation is
obtained.

User data can be targeted to the server. Such data are signaled by the third
highest bit of the flags byte (represented by the mask `0b0010_0000`). All
other data are targeted to all other players who joined the same game.

The payload comprises the user data intended for delivery.

## Protocol Control

Currently, the only type of control datagram is the delivery confirmation
datagram. All bits in the header of these datagrams, except for the first one,
are set to 0. The payload consists of IDs for all user data datagrams that have
been sent reliably and delivered successfully. Each ID is encoded using 3
bytes.
