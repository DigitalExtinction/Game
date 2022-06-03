# Contributing to Digital Extinction

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

## Getting Oriented

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
