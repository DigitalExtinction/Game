use bevy::prelude::*;
use de_gui::{
    ButtonCommands, ButtonOps, GuiCommands, LabelCommands, OuterStyle, TextBoxCommands, ToastEvent,
};
use de_lobby_model::GameMap;
use de_map::hash::MapHash;
use iyes_loopless::prelude::*;

use crate::{
    mapselection::{MapSelectedEvent, SelectMapEvent},
    menu::Menu,
    MenuState,
};

pub(crate) struct CreateGamePlugin;

impl Plugin for CreateGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreateGameEvent>()
            .add_enter_system(MenuState::GameCreation, setup)
            .add_exit_system(MenuState::GameCreation, cleanup)
            .add_system(
                button_system
                    .run_in_state(MenuState::GameCreation)
                    .label(CreateLabel::Buttons),
            )
            .add_system(
                map_selected_system
                    .run_in_state(MenuState::GameCreation)
                    .label(CreateLabel::MapSelected),
            )
            .add_system(
                create_game_system
                    .run_in_state(MenuState::GameCreation)
                    .after(CreateLabel::Buttons)
                    .after(CreateLabel::MapSelected),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum CreateLabel {
    Buttons,
    MapSelected,
}

#[derive(Component, Clone, Copy)]
enum ButtonAction {
    SelectMap,
    Create,
}

#[derive(Resource)]
struct Inputs {
    map: Entity,
}

#[derive(Resource)]
struct SelectedMap(GameMap);

struct CreateGameEvent;

fn setup(mut commands: GuiCommands, menu: Res<Menu>) {
    let column_id = column(&mut commands, menu.root_node());

    let name_row_id = row(&mut commands, column_id);
    text_input(&mut commands, name_row_id, "Name");

    let max_players_row_id = row(&mut commands, column_id);
    text_input(&mut commands, max_players_row_id, "Max Players");

    let map_row_id = row(&mut commands, column_id);
    let map_id = map_button(&mut commands, map_row_id);

    commands.insert_resource(Inputs { map: map_id });

    let buttons_row_id = row(&mut commands, column_id);
    let create_id = commands
        .spawn_button(
            OuterStyle {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                ..default()
            },
            "Create Game",
        )
        .insert(ButtonAction::Create)
        .id();
    commands.entity(buttons_row_id).add_child(create_id);
}

fn column(commands: &mut GuiCommands, parent_id: Entity) -> Entity {
    let column_id = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                size: Size::new(Val::Percent(50.), Val::Percent(100.)),
                margin: UiRect::all(Val::Auto),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .id();
    commands.entity(parent_id).add_child(column_id);
    column_id
}

fn row(commands: &mut GuiCommands, parent_id: Entity) -> Entity {
    let row_id = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                size: Size::new(Val::Percent(100.), Val::Percent(8.)),
                margin: UiRect::new(
                    Val::Percent(0.),
                    Val::Percent(0.),
                    Val::Percent(2.),
                    Val::Percent(2.),
                ),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .id();
    commands.entity(parent_id).add_child(row_id);
    row_id
}

fn text_input(commands: &mut GuiCommands, parent_id: Entity, caption: &str) -> Entity {
    spawn_caption(commands, parent_id, caption);

    let input_id = commands
        .spawn_text_box(
            OuterStyle {
                size: Size::new(Val::Percent(65.), Val::Percent(100.)),
                ..default()
            },
            false,
        )
        .id();
    commands.entity(parent_id).add_child(input_id);
    input_id
}

fn map_button(commands: &mut GuiCommands, parent_id: Entity) -> Entity {
    spawn_caption(commands, parent_id, "Map");

    let input_id = commands
        .spawn_button(
            OuterStyle {
                size: Size::new(Val::Percent(65.), Val::Percent(100.)),
                ..default()
            },
            "-",
        )
        .insert(ButtonAction::SelectMap)
        .id();
    commands.entity(parent_id).add_child(input_id);
    input_id
}

fn spawn_caption(commands: &mut GuiCommands, parent_id: Entity, caption: &str) {
    let caption_id = commands
        .spawn_label(
            OuterStyle {
                size: Size::new(Val::Percent(35.), Val::Percent(100.)),
                ..default()
            },
            caption,
        )
        .id();
    commands.entity(parent_id).add_child(caption_id);
}

fn cleanup(mut commands: GuiCommands) {
    commands.remove_resource::<Inputs>();
    commands.remove_resource::<SelectedMap>();
}

fn button_system(
    interactions: Query<(&Interaction, &ButtonAction), Changed<Interaction>>,
    mut map_events: EventWriter<SelectMapEvent>,
    mut create_events: EventWriter<CreateGameEvent>,
) {
    for (&interaction, &action) in interactions.iter() {
        if let Interaction::Clicked = interaction {
            match action {
                ButtonAction::SelectMap => map_events.send(SelectMapEvent),
                ButtonAction::Create => create_events.send(CreateGameEvent),
            }
        }
    }
}

fn map_selected_system(
    mut commands: Commands,
    mut map_selected_events: EventReader<MapSelectedEvent>,
    intpus: Res<Inputs>,
    mut buttons: ButtonOps,
    mut toasts: EventWriter<ToastEvent>,
) {
    let Some(event) = map_selected_events.iter().last() else { return };
    let hash = match MapHash::try_from(event.path()) {
        Ok(hash) => hash,
        Err(error) => {
            toasts.send(ToastEvent::new(format!("Map error: {error}")));
            return;
        }
    };

    buttons
        .set_text(intpus.map, event.metadata().name().to_owned())
        .unwrap();
    commands.insert_resource(SelectedMap(GameMap::new(
        hash.to_hex(),
        event.metadata().name().to_owned(),
    )));
}

fn create_game_system(
    mut events: EventReader<CreateGameEvent>,
    mut toasts: EventWriter<ToastEvent>,
) {
    // Always exhaust the iterator
    if events.iter().count() == 0 {
        return;
    }

    toasts.send(ToastEvent::new("Not yet implemented."));
}
