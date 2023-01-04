use bevy::prelude::*;
use de_core::state::MenuState;
use de_gui::{ButtonCommands, GuiCommands, OuterStyle};
use iyes_loopless::prelude::*;

use crate::menu::despawn_root_nodes;

pub(crate) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(MenuState::MainMenu, setup)
            .add_exit_system(MenuState::MainMenu, despawn_root_nodes)
            .add_system(button_system.run_in_state(MenuState::MainMenu));
    }
}

#[derive(Component, Clone, Copy)]
struct ButtonAction(MenuState);

fn setup(mut commands: GuiCommands) {
    let root_node = commands
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

    button(
        &mut commands,
        root_node,
        ButtonAction(MenuState::MapSelection),
        "Singleplayer",
    );
    button(
        &mut commands,
        root_node,
        ButtonAction(MenuState::SignIn),
        "Multiplayer",
    );
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
    mut commands: Commands,
    interactions: Query<(&Interaction, &ButtonAction), Changed<Interaction>>,
) {
    for (&interaction, &action) in interactions.iter() {
        if let Interaction::Clicked = interaction {
            commands.insert_resource(NextState(action.0));
        }
    }
}
