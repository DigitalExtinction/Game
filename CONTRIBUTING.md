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

* Try to split your work into separate and atomic pull requests. Put any
  non-obvious reasoning behind any change to the pull request description.
  Separate “preparatory” changes and modifications from new features &
  improvements.

* Do not push any binary files larger than 32KiB directly to the repository,
  use [Git LFS](https://git-lfs.github.com/) instead. For consistency reasons,
  you may track even smaller binary files with Git LFS.

* [Mention](https://docs.github.com/en/issues/tracking-your-work-with-issues/linking-a-pull-request-to-an-issue#linking-a-pull-request-to-an-issue-using-a-keyword)
  relevant issues in your pull request description.

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

* Rustfmt configured by [rustfmt.toml](./rustfmt.toml) is used for Rust code
  formatting. Note that nightly version of the tool is required.

* Maximum width of each line is 100.

* Source lines which are entirely a comment should be limited to 80 characters
  in length (including comment sigils, but excluding indentation) or the
  maximum width of the line (including comment sigils and indentation),
  whichever is smaller.

* By default, make modules, functions, and data structures private. Limit `pub`
  visibility when possible by using `pub(super)` or `pub(crate)` instead of
  plain `pub`.

* Keep individual Bevy system complexity and the number of its parameters low.
  When the complexity grows, split the system and use events for inter-system
  communication.

* SystemSet enum names end with `Set` (for example `CameraSet`). Event struct
  names end with `Event` (for example `DoubleClickEvent`).

* Place Bevy plugins in the same module as their respective events, resources,
  other structs and systems. If a plugin is large, split it into multiple
  smaller plugins instead of dividing it across several modules.

* Decouple individual Bevy plugins as much as possible and limit assumptions
  about the precise functioning of other plugins. Ensure that the
  implementation is resilient to subtle changes in the rest of the codebase.

  For instance, when processing events originating from another plugin, avoid
  making assumptions about their timing or the absence of duplicate or
  accumulated events.

### Non Rust Text Files

* Maximum line width of Markdown files is 79 characters.

### Crate Structure

* Each crate is small (several thousand SLOC or less), simple and
  self-contained.

* The public API of each crate is small or empty.

* Each "Bevy" based crate implements a
  [PluginGroup](https://docs.rs/bevy/latest/bevy/app/trait.PluginGroup.html)
  placed in `lib.rs`.

* The crate is split into one or more individual
  [Plugin](https://docs.rs/bevy/latest/bevy/app/trait.Plugin.html)s. Each is
  placed to a separate module.

* Systems are (usually) located alongside their respective plugins.

* Inter-crate interaction leans heavily on the usage of Bevy events.

* All components and resources exposed in a crate public API, must be inserted,
  removed and modified only from their respective crate.

## Getting Oriented

Rust documentation is automatically build and deployed from `main` branch to
[docs.de-game.org](https://docs.de-game.org/).

The game is split into multiple [crates](/crates), each implementing part of
the game logic. This repository contains a Cargo workspace which consists of
all the sub-crates. The crate inter-dependencies form an orderly DAG.

Topologically sorted crates:

* [de_lobby](/crates/lobby) – lobby server.

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

* [controller](/crates/controller) – handling of user input and containing
  head-up display with 2D in-game UI.

### Repository Structure

* [/assets](/assets) — all game assets are located here. These are distributed
  together with the game executable.

  * [/assets/maps](/assets/maps) — game maps.
  * [/assets/models](/assets/models) — 3D models in [glTF
    2.0](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html) format.
  * [/assets/objects](/assets/objects) — game [object
    definitions](https://docs.de-game.org/objects/).
  * …

* [/crates](/crates) — the game comprises many small crates in a single
  [workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html), all
  of them except the main crate (`de_game`) are located in this directory.

  Each sub-directory contains sources of a crate whose name is the same as the
  name of its directory plus the `de_` prefix.

* [/projects](/projects) — project files (e.g. Blender `.blend` files) used
  during creation of the assets.

* [/docs](/docs) — technical documentation of the game. The documentation is
  automatically extended with Rust docs and deployed to
  [docs.de-game.org](https://docs.de-game.org/) from the `main` branch.

* [/src](/src) — source code of the main crate (`de_game`). All of the game
  functionality is broken down into separate crates. The purpose of the main
  crate is to put everything together via Bevy's plugin groups, thus its source
  code is very compact.

* [/utils](/utils) — various utilities (e.g. small Python scripts) intended for
  contributors.

### Bevy Schedule

See de_core::baseset::GameSet.

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

* 3D models need to be exported into [glTF
  2.0](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html) format.

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
