//! This module translate user input (e.g. mouse events, keyboard presses) into
//! actions.

use bevy::prelude::*;
pub(crate) use executor::{CommandsLabel, GroupAttackEvent, SendSelectedEvent};

use self::{executor::ExecutorPlugin, handlers::HandlersPlugin};

mod executor;
mod handlers;
mod keyboard;

pub(crate) struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(HandlersPlugin).add_plugin(ExecutorPlugin);
    }
}
