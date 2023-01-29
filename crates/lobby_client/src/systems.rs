use bevy::prelude::*;
use de_conf::Configuration;
use de_core::state::AppState;
use de_lobby_model::Token;
use iyes_loopless::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    client::LobbyClient, Authentication, LobbyRequest, ResponseEvent, SignInRequest, SignUpRequest,
};

pub(crate) struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Authentication>()
            .add_system(
                setup_client
                    .track_progress()
                    .run_in_state(AppState::AppLoading),
            )
            .add_system(set_token::<SignInRequest>)
            .add_system(set_token::<SignUpRequest>);
    }
}

fn setup_client(
    mut commands: Commands,
    conf: Option<Res<Configuration>>,
    client: Option<Res<LobbyClient>>,
) -> Progress {
    if client.is_some() {
        return true.into();
    }
    let Some(conf) = conf else { return false.into() };

    let client = LobbyClient::build(conf.multiplayer().server().clone());
    commands.insert_resource(client);
    false.into()
}

fn set_token<T>(mut events: EventReader<ResponseEvent<T>>, mut auth: ResMut<Authentication>)
where
    T: LobbyRequest<Response = Token>,
{
    let Some(event) = events.iter().last() else { return };
    let Ok(token) = event.result() else { return };
    auth.set_token(token.token().to_owned());
}
