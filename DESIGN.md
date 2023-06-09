# Game Design

This documents summarizes high level design of the game. It does not capture
current state but the desired state attainable in reasonable amount of time.

The document is based on conclusions from our [discussions on
ideas](https://github.com/DigitalExtinction/Game/discussions/categories/ideas).
If you have an idea, consult the discussions first.

## Energy

There is only one currency / resource in the game and that is energy.

* Many kinds of activity (for example unit movement, manufacturing &
  construction, combat) require energy. These are shut down by priority if
  there is not enough locally available energy.

* Energy can be generated with several different kinds of power plants: solar,
  nuclear, and wind.

* Energy can be stored in integrated batteries (inside buildings and units) or
  in dedicated battery farms with higher capacity.

* Energy is not a global resource but has to be distributed via an electricity
  grid. The grid consists of Power Hubs transmitting the energy via lasers. The
  grid might be attacked by an opponent. It has to be constructed and defended.

* Energy is transmitted from a source by an energy laser to a target by an
  energy panel. A beam from a laser to a panel is formed along a clear line of
  sight.

* Buildings and units might be equipped with 0 or more energy lasers and energy
  panels.

* The energy grid operates autonomously: it constructs a semi-optimal
  transmission network based on priorities, remaining battery capacities, and
  other criteria.

## Buildings

### Base

* Is capable of producing all unit.
* Has a battery of a very high capacity.
* Is equipped with several laser cannons for defense.
* Produces small amount of energy via integrated solar panels.
* Has several energy lasers.
* Has several energy panels.

### Factory

* Is capable of producing all unit.
* Has a battery of medium capacity.

### Solar Power Plant

* Has a single energy laser.

### Battery Farm

* Stores large amount of energy.
* Has a single energy panel.

### Power Hub

* Has several energy lasers.
* Has several energy panels.
* Has no batteries.

## Units

### Attacker Drone

* Has an integrated battery of medium capacity.
* Has a single energy panel.
* Is equipped with a laser cannon.

### Construction Drone

* Has an integrated battery of small capacity.
* Has a single energy panel.
* Can construct any kind of building.

### Battery Drone

This drone can be used to deliver energy to areas of scarcity.

* Has a battery of large capacity.
* Has a single energy panel.
* Has a single energy laser.
