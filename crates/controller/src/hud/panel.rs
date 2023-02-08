use bevy::prelude::*;
use de_core::{cleanup::DespawnOnGameExit, gamestate::GameState};
use iyes_loopless::prelude::*;

use super::{interaction::InteractionBlocker, HUD_COLOR};

pub(crate) struct PanelPlugin;

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Playing, spawn_details)
            .add_enter_system(GameState::Playing, spawn_action_bar);
    }
}

fn spawn_details(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(20.),
                    height: Val::Percent(30.),
                },
                position_type: PositionType::Absolute,
                position: UiRect::new(
                    Val::Percent(0.),
                    Val::Percent(20.),
                    Val::Percent(70.),
                    Val::Percent(100.),
                ),
                ..default()
            },
            background_color: HUD_COLOR.into(),
            ..default()
        },
        DespawnOnGameExit,
        InteractionBlocker,
    ));
}

fn spawn_action_bar(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(60.),
                    height: Val::Percent(15.),
                },
                position_type: PositionType::Absolute,
                position: UiRect::new(
                    Val::Percent(20.),
                    Val::Percent(80.),
                    Val::Percent(85.),
                    Val::Percent(100.),
                ),
                ..default()
            },
            background_color: HUD_COLOR.into(),
            ..default()
        },
        DespawnOnGameExit,
        InteractionBlocker,
    ));
}
