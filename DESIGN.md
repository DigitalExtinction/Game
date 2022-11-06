# Game Design

This documents summarizes high level design of the game. It does not capture
current state but the desired state attainable in reasonable amount of time.

The document is based on conclusions from our [discussions on
ideas](https://github.com/DigitalExtinction/Game/discussions/categories/ideas).
If you have an idea, consult the discussions first.

## Energy

There is only one currency / resource in the game and that is energy.

* Energy can be generated with several different kinds of power plants: solar,
  nuclear, wind.

* Energy is not a global resource but has to be distributed via an electricity
  grid. The grid consists of high voltage lines (long distance), low voltage
  lines (immediate surroundings) and transformers. Individual parts of the grid
  have a limited maximum transmission capacity. The grid might be attacked by
  an opponent. It has to be constructed and defended.

* Energy can be stored in integrated batteries (inside buildings and units) or
  in dedicated battery farms with higher capacity.

* Many kinds of activity (for example unit movement, manufacturing &
  construction, combat) require energy. These are shut down by priority if
  there is not enough locally available energy.
