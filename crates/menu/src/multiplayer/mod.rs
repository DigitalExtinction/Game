use bevy::prelude::*;
use de_core::nested_state;

use self::{create::CreateGamePlugin, gamelisting::GameListingPlugin, signin::SignInPlugin};
use crate::{menu::ScreenStatePlugin, MenuState};

mod create;
mod gamelisting;
mod requests;
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
        ));
    }
}

nested_state!(
    MenuState::Multiplayer -> MultiplayerState,
    doc = "Each state corresponds to an individual multiplayer related menu screen.",
    enter = multiplayer_entered_system,
    variants = {
        SignIn,
        GameListing,
        GameCreation,
        GameJoined,
    }
);

fn multiplayer_entered_system(mut next_state: ResMut<NextState<MultiplayerState>>) {
    next_state.set(MultiplayerState::SignIn);
}
