use bevy::prelude::*;
use de_core::gresult::GameResult;
use de_gui::{GuiCommands, LabelCommands, OuterStyle};

use crate::{menu::Menu, MenuState};

pub(crate) struct AfterGamePlugin;

impl Plugin for AfterGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::AfterGame), setup)
            .add_systems(OnEnter(MenuState::AfterGame), cleanup);
    }
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<GameResult>();
}

fn setup(mut commands: GuiCommands, menu: Res<Menu>, result: Res<GameResult>) {
    let text = match result.as_ref() {
        GameResult::Finished(result) => {
            if result.won() {
                "You have won!".to_owned()
            } else {
                "You have lost!".to_owned()
            }
        }
        GameResult::Error(message) => {
            error!("Game finished with an error: {message}");
            format!("Error: {message}")
        }
    };
    let text_id = commands.spawn_label(OuterStyle::default(), text).id();
    commands.entity(menu.root_node()).add_child(text_id);
}
