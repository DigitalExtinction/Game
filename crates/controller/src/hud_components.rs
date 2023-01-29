use bevy::prelude::*;

use crate::hud::HudTopVisibleNode;

const HUD_COLOR: Color = Color::BLACK;

pub(crate) fn spawn_details(commands: &mut Commands) {
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

pub(crate) fn spawn_action_bar(commands: &mut Commands) {
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

pub(crate) fn spawn_map(commands: &mut Commands) {
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
