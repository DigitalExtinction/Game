# Digital Extinction

Digital Extinction is a 3D real-time strategy (RTS) game. It is set in the near
future where humans and AI fight over their existence.

It is [open source & free software](#license). Forever! It runs on Linux,
Windows and potentially other platforms.

The game is completely written in [Rust](https://www.rust-lang.org/) with
[Bevy](https://bevyengine.org/) used as the engine.

# Status

The game is still in [early phases](#roadmap) of its development. If you are
looking for a mature game, come back in a few months.

Feedback, bug reports and other [contributions](#contributing) are welcomed.

# How to Play?

[Currently](#status), the only possibility is to build the game from source.

What you need:

* installed and configured Git
* [Git LFS](https://git-lfs.github.com/)
* [Rust](https://www.rust-lang.org/tools/install)

Clone, build & run recipe (will work in majority of the cases):

* `git clone git@github.com:DigitalExtinction/Game.git DigitalExtinction`
* `cd DigitalExtinction`
* make sure that Git LFS files in [assets/](assets/) are pulled
* `cargo run --release`

# Where to Get Help?

Open a question in [Q&A category of this repository's
discussions](https://github.com/DigitalExtinction/Game/discussions/categories/q-a).

# Contributing

Contributions to the project in any form are welcomed. Check out our
[Contributor's Guide](/CONTRIBUTING.md).

# Roadmap

1. [MVP](https://github.com/DigitalExtinction/Game/milestone/1)
1. Release (first) [version
   0.1](https://github.com/DigitalExtinction/Game/milestone/2)
1. Publicly announce the game:
   [#60](https://github.com/DigitalExtinction/Game/issues/60)
1. [Indefinitely improve the game](/CONTRIBUTING.md#development-process)

## MVP

This is an embarrassingly bare bones game. It is complete in the sense that you
can start the game, play against someone and win or loose.

At this stage, the game is too bare bones to be enjoyable. The UI/UX misses
some very important features (like seeing what units are selected, ability to
see health / energy of units, and so on).

The goal of this milestone is to lay down the foundations, setup basic
insfrastrucutre (GitHub Actions, issues labels, …) and achieve an important
psychological milestone.

## 0.1

This is the first published version of the game. In theory, this should be the
earliest possible version, minimizing development time and effort, which is
threshold enjoyable for an actual player.

As opposed to MVP, this version has all the basic UI to make the UX acceptable.
The game mechanics is somewhat expanded so it is no longer thoroughly "dummy".

# License

Digital Extinction is free and open source. All code in this repository is
licensed under GNU GPLv3 ([LICENSE](LICENSE) or
[https://www.gnu.org/licenses/gpl-3.0.en.html](https://www.gnu.org/licenses/gpl-3.0.en.html)).

Unless you explicitly state otherwise, any source code contribution
intentionally submitted for inclusion in the work by you, shall be licensed as
above, without any additional terms or conditions.

All other artwork in this repository, including 3D models, textures,
animations, UI bitmaps, sounds and music is licensed under
[Attribution-ShareAlike 4.0 International (CC BY-SA
4.0)](https://creativecommons.org/licenses/by-sa/4.0/legalcode)
