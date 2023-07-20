use std::ops::Deref;

use bevy::prelude::*;
use de_core::{gresult::GameResult, state::AppState};
use de_gui::ToastEvent;

use crate::{config::NetGameConf, NetState};

pub(super) struct LifecyclePlugin;

impl Plugin for LifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartMultiplayerEvent>()
            .add_event::<ShutdownMultiplayerEvent>()
            .add_event::<FatalErrorEvent>()
            .add_system(cleanup.in_schedule(OnEnter(NetState::None)))
            .add_system(
                finish_game
                    .in_schedule(OnEnter(NetState::None))
                    .run_if(in_state(AppState::InGame)),
            )
            .add_system(
                game_left
                    .in_schedule(OnExit(AppState::InGame))
                    .run_if(not(in_state(NetState::None))),
            )
            .add_system(
                start
                    .run_if(in_state(NetState::None))
                    .run_if(on_event::<StartMultiplayerEvent>()),
            )
            .add_system(
                shutdown
                    .run_if(not(
                        in_state(NetState::None).or_else(in_state(NetState::ShuttingDown))
                    ))
                    .run_if(on_event::<ShutdownMultiplayerEvent>()),
            )
            .add_system(
                errors
                    .run_if(not(in_state(NetState::None)))
                    .run_if(on_event::<FatalErrorEvent>()),
            );
    }
}

/// Send this event to setup and startup multiplayer functionality.
///
/// These events are processed only in [`crate::netstate::NetState::None`]
/// state.
pub struct StartMultiplayerEvent {
    net_conf: NetGameConf,
}

impl StartMultiplayerEvent {
    pub fn new(net_conf: NetGameConf) -> Self {
        Self { net_conf }
    }
}

/// Send this event to shutdown multiplayer functionality.
pub struct ShutdownMultiplayerEvent;

/// Send this event when a fatal multiplayer event occurs. These are events
/// which prevents further continuation of multiplayer game.
///
/// An error will be displayed to the user and multiplayer will shut down.
pub(crate) struct FatalErrorEvent(String);

impl FatalErrorEvent {
    pub(crate) fn new(message: impl ToString) -> Self {
        Self(message.to_string())
    }
}

#[derive(Resource)]
pub(crate) struct NetGameConfRes(NetGameConf);

impl Deref for NetGameConfRes {
    type Target = NetGameConf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<NetGameConfRes>();
}

fn finish_game(mut next_state: ResMut<NextState<AppState>>) {
    info!("Multiplayer terminated during a game. Quitting the game.");
    next_state.set(AppState::InMenu);
}

fn start(
    mut commands: Commands,
    mut next_state: ResMut<NextState<NetState>>,
    mut events: ResMut<Events<StartMultiplayerEvent>>,
) {
    let Some(event) = events.drain().last() else {
        return;
    };

    info!("Multiplayer start event received, initiating multiplayer.");
    commands.insert_resource(NetGameConfRes(event.net_conf));
    next_state.set(NetState::Connecting);
}

fn shutdown(mut next_state: ResMut<NextState<NetState>>) {
    info!("Multiplayer shutdown event received, initiating the shutdown.");
    next_state.set(NetState::ShuttingDown);
}

fn errors(
    mut commands: Commands,
    mut events: EventReader<FatalErrorEvent>,
    mut toasts: EventWriter<ToastEvent>,
    mut shutdowns: EventWriter<ShutdownMultiplayerEvent>,
) {
    let Some(event) = events.iter().next() else {
        return;
    };

    error!("Fatal multiplayer error: {}", event.0);
    toasts.send(ToastEvent::new(&event.0));
    shutdowns.send(ShutdownMultiplayerEvent);

    commands.insert_resource(GameResult::error(&event.0));

    events.clear();
}

fn game_left(mut shutdowns: EventWriter<ShutdownMultiplayerEvent>) {
    info!("Game finished, sending multiplayer shutdown event.");
    shutdowns.send(ShutdownMultiplayerEvent);
}
