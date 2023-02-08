use async_std::path::PathBuf;
use bevy::prelude::*;
use de_core::{gconfig::GameConfig, player::Player, state::AppState};
use de_gui::{ButtonCommands, GuiCommands, OuterStyle, ToastEvent};
use iyes_loopless::prelude::*;

use crate::{
    mapselection::{MapSelectedEvent, SelectMapEvent},
    menu::Menu,
    MenuState,
};

pub(crate) struct SinglePlayerPlugin;

impl Plugin for SinglePlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(MenuState::SinglePlayerGame, setup)
            .add_exit_system(MenuState::SinglePlayerGame, cleanup)
            .add_system(button_system.run_in_state(MenuState::SinglePlayerGame))
            .add_system(map_selected_system.run_in_state(MenuState::SinglePlayerGame));
    }
}

#[derive(Resource)]
struct SelectedMap(Option<PathBuf>);

#[derive(Component, Clone, Copy)]
enum ButtonAction {
    StartGame,
    SelectMap,
}

fn setup(mut commands: GuiCommands, menu: Res<Menu>) {
    commands.insert_resource(SelectedMap(None));

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
        ButtonAction::StartGame,
        "Start Game",
    );
    button(
        &mut commands,
        column_node,
        ButtonAction::SelectMap,
        "Select Map",
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

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<SelectedMap>();
}

fn button_system(
    mut commands: Commands,
    interactions: Query<(&Interaction, &ButtonAction), Changed<Interaction>>,
    map: Res<SelectedMap>,
    mut map_events: EventWriter<SelectMapEvent>,
    mut toasts: EventWriter<ToastEvent>,
) {
    for (&interaction, &action) in interactions.iter() {
        if let Interaction::Clicked = interaction {
            match action {
                ButtonAction::StartGame => match map.0.as_ref() {
                    Some(path) => {
                        commands.insert_resource(GameConfig::new(
                            path,
                            Player::Player1,
                            Player::Player4,
                        ));
                        commands.insert_resource(NextState(AppState::InGame));
                    }
                    None => {
                        toasts.send(ToastEvent::new("No map selected."));
                    }
                },
                ButtonAction::SelectMap => map_events.send(SelectMapEvent),
            };
        }
    }
}

fn map_selected_system(mut events: EventReader<MapSelectedEvent>, mut map: ResMut<SelectedMap>) {
    let Some(event) = events.iter().last() else { return };
    map.0 = Some(event.path().into());
}
