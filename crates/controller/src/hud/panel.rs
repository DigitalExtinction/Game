use bevy::prelude::*;
use de_core::state::GameState;
use iyes_loopless::prelude::*;

use super::interaction::HudTopVisibleNode;

const HUD_COLOR: Color = Color::BLACK;

pub(crate) struct PanelPlugin;

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Playing, spawn_details)
            .add_enter_system(GameState::Playing, spawn_action_bar)
            .add_enter_system(GameState::Playing, spawn_map);
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
        HudTopVisibleNode,
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
        HudTopVisibleNode,
    ));
}

fn spawn_map(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(20.),
                    height: Val::Percent(30.),
                },
                position_type: PositionType::Absolute,
                position: UiRect::new(
                    Val::Percent(80.),
                    Val::Percent(100.),
                    Val::Percent(70.),
                    Val::Percent(100.),
                ),
                ..default()
            },
            background_color: HUD_COLOR.into(),
            ..default()
        },
        HudTopVisibleNode,
    ));
}
