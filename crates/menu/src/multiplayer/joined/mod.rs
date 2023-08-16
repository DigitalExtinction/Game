use bevy::prelude::*;

use self::{state::JoinedGameStatePlugin, ui::JoinedGameUiPlugin};

mod state;
mod ui;

pub(super) struct JoinedGamePlugin;

impl Plugin for JoinedGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((JoinedGameStatePlugin, JoinedGameUiPlugin));
    }
}
