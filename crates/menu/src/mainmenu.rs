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

fn setup(mut commands: GuiCommands) {
    commands.spawn_button(
        OuterStyle {
            size: Size::new(Val::Percent(25.), Val::Percent(10.)),
            margin: UiRect::all(Val::Auto),
        },
        "Start Game",
    );
}

fn button_system(
    mut commands: Commands,
    interactions: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for &interaction in interactions.iter() {
        if let Interaction::Clicked = interaction {
            commands.insert_resource(NextState(MenuState::MapSelection));
        }
    }
}
