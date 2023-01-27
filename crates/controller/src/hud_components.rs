use bevy::asset::Handle;
use bevy::prelude::*;

use de_core::objects::BuildingType;
use de_gui::TextProps;
use crate::hud_interaction::HudButtonAction;

pub(crate) fn spawn_details(
    commands: &mut ChildBuilder,
    font: &TextProps,
) {
    commands.spawn(create_side_node_bundle(Color::BLACK))
        .with_children(|parent| {
            parent.spawn(create_detail_node_bundle(Color::DARK_GRAY, Val::Percent(20.)))
                .with_children(|parent| {
                    parent.spawn(create_text_in_button_bundle(
                        format!("Selection details"),
                        font.font(),
                    ));
                });
            parent.spawn(create_detail_node_bundle(Color::MIDNIGHT_BLUE, Val::Percent(80.)))
                .with_children(|parent| {
                    parent.spawn(create_text_in_button_bundle(
                        format!("Selection units"),
                        font.font(),
                    ));
                });
        });
}

pub(crate) fn spawn_action_bar(
    commands: &mut ChildBuilder,
    font: &TextProps,
) {
    commands.spawn(NodeBundle {
        style: Style {
            size: Size { width: Val::Percent(20.), height: Val::Percent(15.) },
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            align_self: AlignSelf::FlexEnd,
            // align_items: AlignItems::FlexEnd,
            flex_grow: 1.0,
            border: UiRect::top(Val::Px(8.)),
            ..default()
        },
        background_color: BackgroundColor::from(Color::GRAY),
        ..default()
    })
        .with_children(|parent| {
            let key_map = vec![BuildingType::Base, BuildingType::PowerHub];
            key_map
                .iter()
                .for_each(|building_type| {
                    parent
                        .spawn((
                            ButtonBundle {
                                style: Style {
                                    size: Size {
                                        width: Val::Percent(25.),
                                        height: Val::Percent(95.),
                                    },
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    align_self: AlignSelf::Center,
                                    margin: UiRect::new(Val::Percent(0.7), Val::Percent(0.7), Val::Percent(2.5), Val::Percent(2.5)),
                                    ..Style::default()
                                },
                                ..ButtonBundle::default()
                            },
                            HudButtonAction::Build(building_type.clone()),
                        ))
                        .with_children(|parent| {
                            parent.spawn(
                                create_text_in_button_bundle(
                                    format!("Build {:?}", building_type),
                                    font.font(),
                                )
                            );
                        });
                });
        });
}

pub(crate) fn spawn_map(
    commands: &mut ChildBuilder,
    font: &TextProps,
) {
    commands.spawn(create_side_node_bundle(Color::BLACK))
        .with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    size: Size { width: Val::Percent(95.), height: Val::Percent(95.) },
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_self: AlignSelf::Stretch,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Percent(2.5)),
                    ..default()
                },
                background_color: BackgroundColor::from(Color::DARK_GREEN),
                ..default()
            }).with_children(|parent| {
                parent.spawn(create_text_in_button_bundle(
                    format!("Map"),
                    font.font(),
                ));
            });
        });
}

pub fn create_side_node_bundle(color: Color) -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size { width: Val::Percent(20.), height: Val::Percent(30.) },
            justify_content: JustifyContent::FlexStart,
            align_self: AlignSelf::FlexEnd,
            align_items: AlignItems::Stretch,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: BackgroundColor::from(color),
        ..default()
    }
}

pub fn create_detail_node_bundle(color: Color, height: Val) -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size { width: Val::Percent(95.), height },
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_self: AlignSelf::Stretch,
            align_items: AlignItems::FlexStart,
            margin: UiRect::all(Val::Percent(2.5)),
            ..default()
        },
        background_color: BackgroundColor::from(color),
        ..default()
    }
}

pub fn create_text_in_button_bundle(text: String, font: Handle<Font>) -> TextBundle {
    let font_size: f32 = 20.0;
    let color: Color = Color::ALICE_BLUE;
    TextBundle {
        style: Style { ..Style::default() },
        text: Text::from_section(
            text,
            TextStyle {
                font,
                font_size,
                color,
            },
        ).with_alignment(
            TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            },
        ),
        ..TextBundle::default()
    }
}

