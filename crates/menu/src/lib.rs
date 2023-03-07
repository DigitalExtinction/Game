use aftergame::AfterGamePlugin;
use bevy::{app::PluginGroupBuilder, prelude::*};
use create::CreateGamePlugin;
use de_core::{gresult::GameResult, state::AppState};
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
            .add(MenuSetupPlugin)
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

struct MenuSetupPlugin;

impl Plugin for MenuSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<MenuState>()
            .add_system(
                // TODO find a nicer way, i.e. without adding it twice
                apply_state_transition::<MenuState>
                    .in_base_set(CoreSet::StateTransitions)
                    .after(apply_state_transition::<AppState>),
            )
            .add_system(menu_entered_system.in_schedule(OnEnter(AppState::InMenu)))
            .add_system(menu_exited_system.in_schedule(OnExit(AppState::InMenu)));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub(crate) enum MenuState {
    #[default]
    None,
    MainMenu,
    SinglePlayerGame,
    SignIn,
    GameListing,
    GameCreation,
    MultiPlayerGame,
    AfterGame,
}

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

fn menu_exited_system(mut next_state: ResMut<NextState<MenuState>>) {
    next_state.set(MenuState::None);
}
