use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use cleanup::CleanupPlugin;
use iyes_progress::prelude::*;
use stages::StagesPlugin;
use state::{AppState, GameState};
use visibility::VisibilityPlugin;

pub mod assets;
pub mod cleanup;
mod errors;
pub mod events;
pub mod frustum;
pub mod gconfig;
pub mod objects;
pub mod player;
pub mod projection;
pub mod screengeom;
pub mod stages;
pub mod state;
pub mod visibility;

pub struct CorePluginGroup;

impl PluginGroup for CorePluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(ProgressPlugin::new(AppState::AppLoading).continue_to(AppState::InMenu))
            .add(ProgressPlugin::new(GameState::Loading).continue_to(GameState::Playing))
            .add(StagesPlugin)
            .add(VisibilityPlugin)
            .add(CleanupPlugin)
    }
}
