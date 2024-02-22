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

Packages can be transmitted in several reliability modes, as listed below. The
second and third bits (represented by the mask `0b0110_0000`) of the flags byte
control the reliability.

* Unreliable (bits `00`) – these packages may be lost, delivered multiple
  times, or delivered out of order with respect to other packages.
* Unordered (bits `01`) – non-duplicate delivery of these packages is
  guaranteed. However, there are no guarantees regarding the ordering of these
  packages with respect to other packages.
* Semi-ordered (bits `10`) – non-duplicate delivery of these packages is
  guaranteed. Additionally, the packages are guaranteed to be delivered after
  all previously reliably sent packages, that is, all unordered and
  semi-ordered packages sent by the same client before a semi-ordered package
  are always delivered before the semi-ordered package.

Packages can be targeted to the server, as opposed to all other game
participants. This is indicated by the fourth highest bit of the flags byte
(represented by the mask `0b0001_0000`).

Package payload comprises the user data intended for delivery.

## Protocol Control

Currently, the only type of control datagram is the delivery confirmation
datagram. All bits in the header of these datagrams, except for the first one,
are set to 0. The payload consists of IDs for all user data datagrams that have
been sent reliably and delivered successfully. Each ID is encoded using 3
bytes.
