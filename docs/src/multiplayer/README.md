# Multiplayer

The multiplayer functionality comprises a game lobby and in-game communication.

Before a game starts, players are required to log into [DE Lobby
Server](./lobby.md) using their accounts. They can browse through existing
games and join one, or alternatively, create a new game. This functionality and
other functionality is facilitated through a REST API.

When initiating a multiplayer game, all players are interconnected via [DE
Connector](./connector/) game server. All communication is routed through this
server, utilizing a custom real-time networking protocol that operates over
UDP.
