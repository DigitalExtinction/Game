use bevy::prelude::*;
use de_core::gresult::GameResult;
use de_gui::{GuiCommands, LabelCommands, OuterStyle};
use iyes_loopless::prelude::*;

use crate::{menu::Menu, MenuState};

pub(crate) struct AfterGamePlugin;

impl Plugin for AfterGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(MenuState::AfterGame, setup)
            .add_enter_system(MenuState::AfterGame, cleanup);
    }
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<GameResult>();
}

fn setup(mut commands: GuiCommands, menu: Res<Menu>, result: Res<GameResult>) {
    let text = if result.won() {
        "You have won!"
    } else {
        "You have lost! "
    };
    let text_id = commands.spawn_label(OuterStyle::default(), text).id();
    commands.entity(menu.root_node()).add_child(text_id);
}
