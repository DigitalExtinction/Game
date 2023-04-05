use baseset::GameSetsPlugin;
use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use cleanup::CleanupPlugin;
use gamestate::GameStatePlugin;
use iyes_progress::prelude::*;
use state::AppState;
use visibility::VisibilityPlugin;

pub mod assets;
pub mod baseset;
pub mod cleanup;
mod errors;
pub mod events;
pub mod flags;
pub mod frustum;
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
            .add(GameSetsPlugin)
            .add(GameStatePlugin)
            .add(VisibilityPlugin)
            .add(CleanupPlugin)
    }
}
