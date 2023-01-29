use bevy::{app::PluginGroupBuilder, prelude::*};
use de_core::state::AppState;
use gamelisting::GameListingPlugin;
use iyes_loopless::prelude::*;
use iyes_loopless::state::NextState;
use mainmenu::MainMenuPlugin;
use mapselection::MapSelectionPlugin;
use menu::MenuPlugin;
use signin::SignInPlugin;
use singleplayer::SinglePlayerPlugin;

mod gamelisting;
mod mainmenu;
mod mapselection;
mod menu;
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
    }
}

struct MenuSetupPlugin;

impl Plugin for MenuSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(MenuState::None)
            .add_enter_system(AppState::InMenu, menu_entered_system);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum MenuState {
    None,
    MainMenu,
    SinglePlayerGame,
    SignIn,
    GameListing,
}

fn menu_entered_system(mut commands: Commands) {
    commands.insert_resource(NextState(MenuState::MainMenu));
}
