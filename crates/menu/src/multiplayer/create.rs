use bevy::prelude::*;
use de_gui::{
    ButtonCommands, ButtonOps, GuiCommands, LabelCommands, OuterStyle, TextBoxCommands,
    TextBoxQuery, ToastEvent,
};
use de_lobby_model::{GameConfig, GameMap, Validatable};
use de_map::hash::MapHash;

use super::{setup::SetupGameEvent, MultiplayerState};
use crate::{
    mapselection::{MapSelectedEvent, SelectMapEvent},
    menu::Menu,
};

pub(super) struct CreateGamePlugin;

impl Plugin for CreateGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreateGameEvent>()
            .add_systems(OnEnter(MultiplayerState::GameCreation), setup)
            .add_systems(OnExit(MultiplayerState::GameCreation), cleanup)
            .add_systems(
                Update,
                (
                    button_system.in_set(CreateSet::Buttons),
                    map_selected_system.in_set(CreateSet::MapSelected),
                    create_game_system
                        .run_if(on_event::<CreateGameEvent>())
                        .after(CreateSet::Buttons)
                        .after(CreateSet::MapSelected),
                )
                    .run_if(in_state(MultiplayerState::GameCreation)),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
enum CreateSet {
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
    name: Entity,
    max_players: Entity,
    map: Entity,
}

#[derive(Resource)]
struct SelectedMap(GameMap);

#[derive(Event)]
struct CreateGameEvent;

fn setup(mut commands: GuiCommands, menu: Res<Menu>) {
    let column_id = column(&mut commands, menu.root_node());

    let name_row_id = row(&mut commands, column_id);

    let name_id = text_input(&mut commands, name_row_id, "Name");

    let max_players_row_id = row(&mut commands, column_id);
    let max_players_id = text_input(&mut commands, max_players_row_id, "Max Players");

    let map_row_id = row(&mut commands, column_id);
    let map_id = map_button(&mut commands, map_row_id);

    commands.insert_resource(Inputs {
        name: name_id,
        max_players: max_players_id,
        map: map_id,
    });

    let buttons_row_id = row(&mut commands, column_id);
    let create_id = commands
        .spawn_button(
            OuterStyle {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
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
                width: Val::Percent(50.),
                height: Val::Percent(100.),
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
                width: Val::Percent(100.),
                height: Val::Percent(8.),
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
                width: Val::Percent(65.),
                height: Val::Percent(100.),
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
                width: Val::Percent(65.),
                height: Val::Percent(100.),
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
                width: Val::Percent(35.),
                height: Val::Percent(100.),
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
        if let Interaction::Pressed = interaction {
            match action {
                ButtonAction::SelectMap => {
                    map_events.send(SelectMapEvent);
                }
                ButtonAction::Create => {
                    create_events.send(CreateGameEvent);
                }
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
    let Some(event) = map_selected_events.read().last() else {
        return;
    };
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
    inputs: Res<Inputs>,
    texts: TextBoxQuery,
    selected_map: Option<Res<SelectedMap>>,
    mut toasts: EventWriter<ToastEvent>,
    mut setup_events: EventWriter<SetupGameEvent>,
) {
    let Some(selected_map) = selected_map else {
        toasts.send(ToastEvent::new("No map selected."));
        return;
    };

    let name = texts.text(inputs.name).unwrap().to_string();
    let max_players: u8 = match texts.text(inputs.max_players).unwrap().parse() {
        Ok(value) => value,
        Err(error) => {
            toasts.send(ToastEvent::new(format!("Invalid max players: {error}")));
            return;
        }
    };

    let game_config = GameConfig::new(name, max_players, selected_map.0.clone());
    if let Err(error) = game_config.validate() {
        toasts.send(ToastEvent::new(format!("{error}")));
        return;
    }
    setup_events.send(SetupGameEvent::new(game_config));
}
