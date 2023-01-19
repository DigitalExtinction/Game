use core::fmt;

use bevy::{app::AppExit, prelude::*};
use de_core::{stages::GameStage, state::GameState};
use de_gui::{ButtonCommands, GuiCommands, OuterStyle};
use iyes_loopless::prelude::*;

pub(crate) struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ToggleGameMenu>()
            .add_enter_system(GameState::Playing, setup)
            .add_exit_system(GameState::Playing, cleanup)
            .add_system_set_to_stage(
                GameStage::Input,
                SystemSet::new()
                    .with_system(
                        toggle_system
                            .run_in_state(GameState::Playing)
                            .label(GameMenuLabel::Toggle),
                    )
                    .with_system(button_system.run_in_state(GameState::Playing)),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum GameMenuLabel {
    Toggle,
}

pub(crate) struct ToggleGameMenu;

#[derive(Component)]
struct PopUpMenu;

#[derive(Component, Clone, Copy)]
enum ButtonAction {
    Quit,
}

impl fmt::Display for ButtonAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Quit => write!(f, "Quit Game"),
        }
    }
}

fn setup(mut commands: GuiCommands) {
    let root_node = commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            visibility: Visibility::INVISIBLE,
            ..default()
        })
        .insert(PopUpMenu)
        .id();

    let menu_node = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                size: Size::new(Val::Percent(25.), Val::Percent(50.)),
                padding: UiRect::horizontal(Val::Percent(1.)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::BLACK.into(),
            ..default()
        })
        .id();
    commands.entity(root_node).add_child(menu_node);

    button(&mut commands, menu_node, ButtonAction::Quit);
}

fn button(commands: &mut GuiCommands, parent: Entity, action: ButtonAction) {
    let button = commands
        .spawn_button(
            OuterStyle {
                size: Size::new(Val::Percent(100.), Val::Percent(16.)),
                margin: UiRect::new(
                    Val::Percent(0.),
                    Val::Percent(0.),
                    Val::Percent(2.),
                    Val::Percent(2.),
                ),
            },
            format!("{action}"),
        )
        .insert(action)
        .id();
    commands.entity(parent).add_child(button);
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<PopUpMenu>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn toggle_system(
    mut events: EventReader<ToggleGameMenu>,
    mut query: Query<&mut Visibility, With<PopUpMenu>>,
) {
    if events.iter().count() % 2 == 0 {
        return;
    }
    query.single_mut().toggle();
}

fn button_system(
    mut exit: EventWriter<AppExit>,
    interactions: Query<(&Interaction, &ButtonAction), Changed<Interaction>>,
) {
    for (&interaction, &action) in interactions.iter() {
        if let Interaction::Clicked = interaction {
            match action {
                ButtonAction::Quit => exit.send(AppExit),
            }
        }
    }
}
