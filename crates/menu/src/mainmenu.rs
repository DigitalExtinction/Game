use bevy::{app::AppExit, prelude::*};
use de_gui::{ButtonCommands, GuiCommands, OuterStyle};

use crate::{menu::Menu, MenuState};

pub(crate) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(MenuState::MainMenu)))
            .add_system(button_system.run_if(in_state(MenuState::MainMenu)));
    }
}

#[derive(Component, Clone, Copy)]
enum ButtonAction {
    SwithState(MenuState),
    Quit,
}

fn setup(mut commands: GuiCommands, menu: Res<Menu>) {
    let column_node = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                size: Size::new(Val::Percent(25.), Val::Percent(100.)),
                margin: UiRect::all(Val::Auto),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .id();
    commands.entity(menu.root_node()).add_child(column_node);

    button(
        &mut commands,
        column_node,
        ButtonAction::SwithState(MenuState::SinglePlayerGame),
        "Singleplayer",
    );
    button(
        &mut commands,
        column_node,
        ButtonAction::SwithState(MenuState::SignIn),
        "Multiplayer",
    );
    button(&mut commands, column_node, ButtonAction::Quit, "Quit Game");
}

fn button(commands: &mut GuiCommands, parent: Entity, action: ButtonAction, caption: &str) {
    let button = commands
        .spawn_button(
            OuterStyle {
                size: Size::new(Val::Percent(100.), Val::Percent(8.)),
                margin: UiRect::new(
                    Val::Percent(0.),
                    Val::Percent(0.),
                    Val::Percent(2.),
                    Val::Percent(2.),
                ),
            },
            caption,
        )
        .insert(action)
        .id();
    commands.entity(parent).add_child(button);
}

fn button_system(
    mut next_state: ResMut<NextState<MenuState>>,
    mut exit: EventWriter<AppExit>,
    interactions: Query<(&Interaction, &ButtonAction), Changed<Interaction>>,
) {
    for (&interaction, &action) in interactions.iter() {
        if let Interaction::Clicked = interaction {
            match action {
                ButtonAction::SwithState(state) => next_state.set(state),
                ButtonAction::Quit => exit.send(AppExit),
            };
        }
    }
}
