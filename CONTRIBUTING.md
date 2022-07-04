# Contributing to Digital Extinction

## Suggesting Changes

1. Start an informal
   [discussions](https://github.com/DigitalExtinction/Game/discussions).
1. Open a formal [issue](https://github.com/DigitalExtinction/Game/issues).

## Making Changes

1. First, discuss any non-trivial or potentially controversial changes in our
   [Discussions](https://github.com/DigitalExtinction/Game/discussions).

   You can skip this step if you are basing you changes on an already concluded
   discussion or an [issue](https://github.com/DigitalExtinction/Game/issues).

1. Implement the changes. Do not forget to include appropriate unit tests and,
   when possible, thoroughly test you changes manually.

1. Open a pull request (PR).

1. [@Indy2222](https://github.com/Indy2222) and the community review the PR.
   During the review process, the PR might be accepted right away, changes
   might be requested or it might be rejected.

## Development Process

* `main` branch is considered stable. In theory, it should be possible to
  release a new version from the branch at any time.

* The development process is incremental. There are no final or big versions.

* Release train: after version
  [0.1](https://github.com/DigitalExtinction/Game/milestone/2) is released, a
  new version is released every 6 weeks.

## Coding Style

* We are limiting the amount of unsafe code to the very minimum. The same
  principle applies to complex architecture or code patterns.

  A lot of the ergonomy and performance gains unlocked by unsafe or complex
  code are already delivered by Bevy and its API.

## Getting Oriented

The game is split into multiple [crates](/crates), each implementing part of
the game logic. This repository contains a Cargo workspace which consists of
all the sub-crates.

The intention is:

* each crate is small, simple and self contained,
* the public API of each crate is small or empty,
* most of the crates expose a Bevy PluginGroup – inter-crate interaction is
  handled via Bevy's ECS & events,
* the crate inter-dependencies form an orderly DAG.

Topologically sorted crates:

* [core](/crates/core) – various simple core utilities, structs, and so on.
  These are used across many different crates.

* [map](/crates/map) – map (de)serialization, validation and representation
  functionalities.

* [terrain](/crates/terrain) – functionality related to game terrain.

* [objects](/crates/objects) – caching of object on the game map.

* [index](/crates/index) – spatial index of all solid entities in the game.

* [spawner](/crates/spawner) – object spawning, drafting and construction.

* [camera](/crates/camera)

* [loader](/crates/loader) – map loading logic.

* [pathing](/crates/pathing) – global path finding and patch (re)scheduling.

* [movement](/crates/movement) – entity movement, local dynamic obstacle
  avoidance, kinematics and similar.

* [behaviour](/crates/behaviour) – unit level behaviour and AI.

* [controller](/crates/controller) – handling of user input.

### Coordinate Systems

3D XYZ world coordinates are right handed. Mean sea level (MSL) plane lies on
XZ axes. Y axis points upwards.

2D map coordinates X (longitude) Y (latitude) map to 3D world coordinates X and
-Z respectively. Always use module
[de_core::projection](/crates/core/src/projection.rs) for projection onto MSL
plane or conversion between 3D world coordinates and 2D map coordinates.

### Geometry & Linear Algebra

[Glam](https://github.com/bitshifter/glam-rs) is used as the primary linear
algebra crate.

[Parry](https://github.com/dimforge/parry) is used for collision detection and
geometry. Parry is using [Nalgebra](https://github.com/dimforge/nalgebra) for
linear algebra, thus Nalgebra is used in various places where usage of Glam
would lead to frequent back and forth conversions.
