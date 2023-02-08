use bevy::{app::PluginGroupBuilder, prelude::*};
use create::CreateGamePlugin;
use de_core::state::AppState;
use gamelisting::GameListingPlugin;
use iyes_loopless::prelude::*;
use iyes_loopless::state::NextState;
use mainmenu::MainMenuPlugin;
use mapselection::MapSelectionPlugin;
use menu::MenuPlugin;
use signin::SignInPlugin;
use singleplayer::SinglePlayerPlugin;

mod create;
mod gamelisting;
mod mainmenu;
mod mapselection;
mod menu;
mod requests;
mod signin;
mod singleplayer;

pub struct MenuPluginGroup;

impl PluginGroup for MenuPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(MenuSetupPlugin)
            .add(MenuPlugin)
            .add(MainMenuPlugin)
            .add(MapSelectionPlugin)
            .add(SignInPlugin)
            .add(GameListingPlugin)
            .add(SinglePlayerPlugin)
            .add(CreateGamePlugin)
    }
}

struct MenuSetupPlugin;

impl Plugin for MenuSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state_before_stage(CoreStage::PreUpdate, MenuState::None)
            .add_enter_system(AppState::InMenu, menu_entered_system)
            .add_exit_system(AppState::InMenu, menu_exited_system);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum MenuState {
    None,
    MainMenu,
    SinglePlayerGame,
    SignIn,
    GameListing,
    GameCreation,
    MultiPlayerGame,
}

fn menu_entered_system(mut commands: Commands) {
    commands.insert_resource(NextState(MenuState::MainMenu));
}

fn menu_exited_system(mut commands: Commands) {
    commands.insert_resource(NextState(MenuState::None));
}
