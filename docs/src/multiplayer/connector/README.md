# DE Connector

DE Connector interconnects players so they can exchange messages in real-time.

Communication between clients and the server relies on a [low-level
communication protocol](./protocol.md), providing reliable and non-reliable
means of binary data exchange.

[Binary encoded messages](./encoding.md) are sent via the low-level protocol as
[packages](./protocol.md#package) to the server. Based on the package target
(peers), the server either receives and interprets the data, or transmits it to
other clients connected to the same game.

Game participants exchange [messages](./encoding.md) through a game server. For
efficiency, multiple messages may be grouped into a single
[package](./protocol.md#package), and the messages are buffered and flushed at
the end of each game update (frame). The buffering is independent for
unreliable, unordered, and semi-ordered packages, with their buffers being
flushed in the respective order.

The main server listens on port 8082, which is designated solely for server
control messages such as game initiation requests. Upon the creation of a new
game, a unique sub-server, listening on a different port, is started. It is
within these sub-servers that clients exchange data among themselves.

## Principles

Game networking is designed in such a way that complete game determinism is not
required for certain aspects, such as entity movement. It is assumed, however,
that the divergence among individual clients is negligible in the short term
(on the order of several seconds). This approach simplifies movement simulation
but comes at the cost of the need for regular synchronization and higher
network bandwidth requirements.

All clients simulate all game activities while receiving updates to correct any
possible simulation divergence. Each action has its authoritative owner,
corresponding to the client that is locally controlling the entity responsible
for the action, whether through a human player or AI. The owner sends a
synchronization message, which is respected by the other clients. For instance,
an entity damage action (a decrease in health) is owned by the client
simulating the entity that causes the damage (for example, by firing a laser
cannon).

Whenever possible, messages are sent in unreliable mode. This method of data
exchange consumes fewer resources but does not guarantee delivery or ordering.
For instance, entity movement synchronization occurs unreliably. This approach
is both possible and preferable because the position of each entity, out of
possibly thousands, is regularly updated, and messages that reference
non-existent entities are disregarded.
