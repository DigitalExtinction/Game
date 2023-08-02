use bevy::prelude::*;
use de_core::nested_state;
use de_multiplayer::MultiplayerShuttingDownEvent;

use self::{
    create::CreateGamePlugin, gamelisting::GameListingPlugin, joined::JoinedGamePlugin,
    setup::SetupGamePlugin, signin::SignInPlugin,
};
use crate::{menu::ScreenStatePlugin, MenuState};

mod create;
mod gamelisting;
mod joined;
mod requests;
mod setup;
mod signin;

pub(super) struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MultiplayerStatePlugin,
            ScreenStatePlugin::<MultiplayerState>::default(),
            SignInPlugin,
            GameListingPlugin,
            CreateGamePlugin,
            SetupGamePlugin,
            JoinedGamePlugin,
        ))
        .add_systems(
            PostUpdate,
            go_to_sign_in
                .run_if(in_state(MenuState::Multiplayer))
                .run_if(on_event::<MultiplayerShuttingDownEvent>()),
        );
    }
}

nested_state!(
    MenuState::Multiplayer -> MultiplayerState,
    doc = "Each state corresponds to an individual multiplayer related menu screen.",
    enter = go_to_sign_in,
    variants = {
        SignIn,
        GameListing,
        GameCreation,
        GameSetup,
        GameJoined,
    }
);

fn go_to_sign_in(mut next_state: ResMut<NextState<MultiplayerState>>) {
    next_state.set(MultiplayerState::SignIn);
}
