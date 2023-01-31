//! This module translate user input (e.g. mouse events, keyboard presses) into
//! actions.

use bevy::prelude::*;

use self::handlers::HandlersPlugin;

mod handlers;
mod keyboard;

pub(crate) struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(HandlersPlugin);
    }
}
