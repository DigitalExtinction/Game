use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use iyes_progress::prelude::*;
use state::GameState;

pub mod assets;
mod errors;
pub mod events;
pub mod gconfig;
pub mod objects;
pub mod player;
pub mod projection;
pub mod state;

pub struct CorePluginGroup;

impl PluginGroup for CorePluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(ProgressPlugin::new(GameState::Loading).continue_to(GameState::Playing));
    }
}
