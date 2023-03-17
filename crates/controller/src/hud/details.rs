use bevy::prelude::*;
use de_core::{cleanup::DespawnOnGameExit, gamestate::GameState};

use super::{interaction::InteractionBlocker, HUD_COLOR};

pub(crate) struct DetailsPlugin;

impl Plugin for DetailsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_details.in_schedule(OnEnter(GameState::Playing)));
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
