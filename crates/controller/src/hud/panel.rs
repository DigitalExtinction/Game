use bevy::prelude::*;
use de_core::state::GameState;
use de_gui::{GuiCommands, LabelCommands, OuterStyle};
use iyes_loopless::prelude::*;

use super::interaction::InteractionBlocker;

const HUD_COLOR: Color = Color::BLACK;

pub(crate) struct PanelPlugin;

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Playing, spawn_details)
            .add_enter_system(GameState::Playing, spawn_action_bar)
            .add_enter_system(GameState::Playing, spawn_map);
    }
}

fn spawn_details(mut commands: GuiCommands) {
    let parent = commands
        .spawn((
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
                    padding: UiRect::all(Val::Percent(1.)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: HUD_COLOR.into(),
                ..default()
            },
            InteractionBlocker,
        ))
        .id();
    let caption = commands
        .spawn_label(
            OuterStyle {
                size: Size::new(Val::Percent(100.), Val::Percent(20.)),
                ..default()
            },
            format!("Object Detail"),
        )
        .id();
    commands.entity(parent).add_child(caption);
    let details = commands
        .spawn(NodeBundle {
            style: Style {
                position: UiRect::all(Val::Percent(0.)),
                size: Size::new(Val::Percent(100.), Val::Percent(80.)),
                border: UiRect::all(Val::Px(2.)),
                ..default()
            },
            background_color: Color::DARK_GRAY.into(),
            ..default()
        })
        .id();
    commands.entity(parent).add_child(details);
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
        InteractionBlocker,
    ));
}

fn spawn_map(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
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
                padding: UiRect::all(Val::Percent(1.)),
                ..default()
            },
            background_color: HUD_COLOR.into(),
            ..default()
        })
        .insert(InteractionBlocker)
        .with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    position: UiRect::all(Val::Percent(0.)),
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    ..default()
                },
                background_color: Color::BISQUE.into(),
                ..default()
            });
        });
}
