use bevy::prelude::*;
use de_core::{
    gconfig::GameConfig,
    player::Player,
    state::{AppState, GameState, MenuState},
};
use iyes_loopless::prelude::*;

use crate::menu::{despawn_root_nodes, Text};

pub(crate) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(MenuState::MainMenu, setup)
            .add_exit_system(MenuState::MainMenu, despawn_root_nodes)
            .add_system(button_system.run_in_state(MenuState::MainMenu));
    }
}

fn setup(mut commands: Commands, text: Res<Text>) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Percent(25.), Val::Percent(10.)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Start Game",
                text.button_text_style(),
            ));
        });
}

fn button_system(
    mut commands: Commands,
    interactions: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for &interaction in interactions.iter() {
        if let Interaction::Clicked = interaction {
            commands.insert_resource(GameConfig::new("maps/huge.dem.tar", Player::Player1));
            commands.insert_resource(NextState(MenuState::None));
            commands.insert_resource(NextState(AppState::InGame));
            commands.insert_resource(NextState(GameState::Loading));
        }
    }
}
