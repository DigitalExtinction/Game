use aftergame::AfterGamePlugin;
use bevy::{app::PluginGroupBuilder, prelude::*};
use create::CreateGamePlugin;
use de_core::{gresult::GameResult, nested_state, state::AppState};
use gamelisting::GameListingPlugin;
use mainmenu::MainMenuPlugin;
use mapselection::MapSelectionPlugin;
use menu::MenuPlugin;
use signin::SignInPlugin;
use singleplayer::SinglePlayerPlugin;

mod aftergame;
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
            .add(MenuStatePlugin)
            .add(MenuPlugin)
            .add(MainMenuPlugin)
            .add(MapSelectionPlugin)
            .add(SignInPlugin)
            .add(GameListingPlugin)
            .add(SinglePlayerPlugin)
            .add(CreateGamePlugin)
            .add(AfterGamePlugin)
    }
}

nested_state!(
    AppState::InMenu -> MenuState,
    doc = "Top-level menu state. Each variant corresponds to a single menu screen.",
    enter = menu_entered_system,
    variants = {
        MainMenu,
        SinglePlayerGame,
        SignIn,
        GameListing,
        GameCreation,
        MultiPlayerGame,
        AfterGame,
    }
);

fn menu_entered_system(
    mut next_state: ResMut<NextState<MenuState>>,
    result: Option<Res<GameResult>>,
) {
    if result.is_some() {
        next_state.set(MenuState::AfterGame);
    } else {
        next_state.set(MenuState::MainMenu);
    }
}
