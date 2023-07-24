//! This module extends default Bevy schedules.

use bevy::{app::MainScheduleOrder, ecs::schedule::ScheduleLabel, prelude::*};

pub struct GameSchedulesPlugin;

impl Plugin for GameSchedulesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut main: ResMut<MainScheduleOrder>) {
    main.insert_after(First, InputSchedule);
    main.insert_after(InputSchedule, PreMovement);
    main.insert_after(PreMovement, Movement);
    main.insert_after(Movement, PostMovement);
}

/// All user input is handled during this schedule.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InputSchedule;

/// The game state is prepared for movement stage during this schedule. The
/// preparation includes, among other things, global path finding & planning
/// related updates.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PreMovement;

/// All of "game active" entity movement (changes to
/// [`bevy::prelude::Transform`]) happens during this schedule (an in no other
/// stage).
///
/// "Game active" entities are those which impact the game dynamics. For
/// example buildings, units or the terrain. Auxiliary entities, for example
/// building drafts, might be moved during different schedules.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Movement;

/// This schedule includes for example update to spatial index of movable objects.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PostMovement;
