use aftergame::AfterGamePlugin;
use bevy::{app::PluginGroupBuilder, prelude::*};
use de_core::{gresult::GameResult, nested_state, state::AppState};
use mainmenu::MainMenuPlugin;
use mapselection::MapSelectionPlugin;
use menu::{MenuPlugin, ScreenStatePlugin};
use multiplayer::MultiplayerPlugin;
use singleplayer::SinglePlayerPlugin;

mod aftergame;
mod mainmenu;
mod mapselection;
mod menu;
mod multiplayer;
mod singleplayer;

pub struct MenuPluginGroup;

impl PluginGroup for MenuPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(MenuStatePlugin)
            .add(MenuPlugin)
            .add(ScreenStatePlugin::<MenuState>::default())
            .add(MainMenuPlugin)
            .add(MapSelectionPlugin)
            .add(SinglePlayerPlugin)
            .add(MultiplayerPlugin)
            .add(AfterGamePlugin)
    }
}

nested_state!(
    AppState::InMenu -> MenuState,
    doc = "Top-level menu state. Each variant corresponds to menu section or a single menu screen.",
    enter = menu_entered_system,
    variants = {
        MainMenu,
        SinglePlayerGame,
        Multiplayer,
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
