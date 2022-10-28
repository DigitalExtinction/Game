use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use iyes_progress::prelude::*;
use stages::StagesPlugin;
use state::GameState;

pub mod assets;
mod errors;
pub mod events;
pub mod frustum;
pub mod gconfig;
pub mod objects;
pub mod player;
pub mod projection;
pub mod stages;
pub mod state;

pub struct CorePluginGroup;

impl PluginGroup for CorePluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(ProgressPlugin::new(GameState::Loading).continue_to(GameState::Playing))
            .add(StagesPlugin);
    }
}
