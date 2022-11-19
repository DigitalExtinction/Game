# Contributing to Digital Extinction

## Suggesting Changes

1. Start an informal
   [discussions](https://github.com/DigitalExtinction/Game/discussions).
1. Open a formal [issue](https://github.com/DigitalExtinction/Game/issues).

## First Time Contributions

* Look for already opened issues marked with the
  [E-Good-First-Issue](https://github.com/DigitalExtinction/Game/labels/E-Good-First-Issue)
  label.

## Making Changes

1. First, discuss any non-trivial or potentially controversial changes in our
   [Discussions](https://github.com/DigitalExtinction/Game/discussions).

   You can skip this step if you are basing you changes on an already concluded
   discussion or an [issue](https://github.com/DigitalExtinction/Game/issues).

1. Currently, the surface area of the game is still relatively small. To avoid
   duplicate work while working on non-trivial changes, please mention your
   intention to do the work in the appropriate issue.

   You may consult [Indy's
   Backlog](https://github.com/orgs/DigitalExtinction/projects/2) to see what
   he intends to work on in the near future.

1. Implement the changes. Do not forget to include appropriate unit tests and,
   when possible, thoroughly test you changes manually.

1. Open a pull request (PR).

1. [@Indy2222](https://github.com/Indy2222) and the community review the PR.
   During the review process, the PR might be accepted right away, changes
   might be requested or it might be rejected.

## Pull Requests & Git

* Try to split your work into separate and atomic commits. Put any non-obvious
  reasoning behind any change to the commit description. Separate “preparatory”
  changes and modifications from new features & improvements. [Hide the sausage
  making](https://sethrobertson.github.io/GitBestPractices/#sausage).

* Do not push any binary files larger than 32KiB directly to the repository,
  use [Git LFS](https://git-lfs.github.com/) instead. For consistency reasons,
  you may track even smaller binary files with Git LFS.

* [Link](https://docs.github.com/en/issues/tracking-your-work-with-issues/linking-a-pull-request-to-an-issue#linking-a-pull-request-to-an-issue-using-a-keyword)
  relevant issues from your commits.

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

Rust documentation is automatically build and deployed from `main` branch to
[docs.de-game.org](https://docs.de-game.org/).

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

* [de_uom](/crates/uom) – type safe units of measurements.

* [core](/crates/core) – various simple core utilities, structs, and so on.
  These are used across many different crates.

* [map](/crates/map) – map (de)serialization, validation and representation
  functionalities.

* [menu](/crates/menu) – game menu and related functionality.

* [terrain](/crates/terrain) – functionality related to game terrain.

* [objects](/crates/objects) – caching of object on the game map.

* [index](/crates/index) – spatial index of all solid entities in the game.

* [signs](/crates/signs) – various world space UI signs in Digital Extinction.
  For example health bars, arrows, and so on.

* [spawner](/crates/spawner) – object spawning, drafting and construction.

* [camera](/crates/camera)

* [loader](/crates/loader) – map loading logic.

* [pathing](/crates/pathing) – global path finding and path (re)scheduling.

* [movement](/crates/movement) – entity movement, local dynamic obstacle
  avoidance, kinematics and similar.

* [behaviour](/crates/behaviour) – unit level behaviour and AI.

* [combat](/crates/combat) – attacking, projectile & laser simulation and
  similar.

* [ui](/crates/ui) – 2D in game UI.

* [controller](/crates/controller) – handling of user input.

### Bevy Schedule Stages

See de_core::stages::GameStage.

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

## Contributing Assets

### 3D Models

* See the [A-Models](https://github.com/DigitalExtinction/Game/labels/A-Models)
  issue label to filter issues related to the 3D models.

* 3D models (buildings, units, map elements, etc.) are located in
  [/assets/models](/assets/models).

* 3D models need to be exported into glTF format.

* Whenever applicable and desirable by the author, add the original 3D modeling
  project file (e.g. .blend file from Blender) to `/projects/` directory.

* The +Y axis points upwards, and the front of the model points in the +X
  direction.

* The ground consists of the XZ plane (Y = 0) and is located just below the
  model. Foundations of buildings may go below the ground plane to allow
  construction on uneven terrain. The altitude of flying units is controlled
  independently of the model, thus even flying objects should be just above the
  ground plane (and not higher).

* Whenever applicable, models are centered in the XZ axes. I.e. axis-aligned
  bounding box (AABB) of the model has its center at (X=0, Y=?, Z=0).
