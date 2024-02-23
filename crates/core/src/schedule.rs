//! This module extends default Bevy schedules.

use bevy::{
    app::MainScheduleOrder,
    ecs::schedule::{ScheduleBuildSettings, ScheduleLabel},
    prelude::*,
};

pub struct GameSchedulesPlugin;

impl GameSchedulesPlugin {
    fn insert_schedule(
        app: &mut App,
        after: impl ScheduleLabel,
        schedule_label: impl ScheduleLabel + Clone,
    ) {
        let mut schedule = Schedule::new(schedule_label.clone());
        schedule.set_build_settings(ScheduleBuildSettings {
            auto_insert_apply_deferred: false,
            ..default()
        });
        app.add_schedule(schedule);
        let mut main_schedule_order = app.world.resource_mut::<MainScheduleOrder>();
        main_schedule_order.insert_after(after, schedule_label);
    }
}

impl Plugin for GameSchedulesPlugin {
    fn build(&self, app: &mut App) {
        Self::insert_schedule(app, First, InputSchedule);
        Self::insert_schedule(app, InputSchedule, PreMovement);
        Self::insert_schedule(app, PreMovement, Movement);
        Self::insert_schedule(app, Movement, PostMovement);
    }
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
