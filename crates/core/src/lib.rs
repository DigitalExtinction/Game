use schedule::GameSchedulesPlugin;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use cleanup::CleanupPlugin;
use gamestate::GameStateSetupPlugin;
use iyes_progress::prelude::*;
use state::AppState;
use visibility::VisibilityPlugin;

pub mod assets;
pub mod schedule;
pub mod cleanup;
mod errors;
pub mod events;
pub mod flags;
pub mod frustum;
pub mod fs;
pub mod gamestate;
pub mod gconfig;
pub mod gresult;
pub mod objects;
pub mod player;
pub mod projection;
pub mod screengeom;
pub mod state;
pub mod transition;
pub mod vecord;
pub mod visibility;

pub struct CorePluginGroup;

impl PluginGroup for CorePluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(ProgressPlugin::new(AppState::AppLoading).continue_to(AppState::InMenu))
            .add(GameSchedulesPlugin)
            .add(GameStateSetupPlugin)
            .add(VisibilityPlugin)
            .add(CleanupPlugin)
    }
}
