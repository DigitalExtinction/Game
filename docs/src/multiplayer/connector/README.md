# DE Connector

DE Connector interconnects players so they can exchange messages in real-time.

Communication between clients and the server relies on a [low-level
communication protocol](./protocol.md), providing reliable and non-reliable
means of binary data exchange.

[Binary encoded messages](./encoding.md) are sent via the low-level protocol to
the server. Based on the data target (peers), the server either receives and
interprets the data, or transmits it to other clients connected to the same
game.

The main server listens on port 8082, which is designated solely for server
control messages such as game initiation requests. Upon the creation of a new
game, a unique sub-server, listening on a different port, is started. It is
within these sub-servers that clients exchange data among themselves.
