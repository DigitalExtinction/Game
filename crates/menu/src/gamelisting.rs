use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch};
use de_gui::{ButtonCommands, GuiCommands, LabelCommands, OuterStyle, ToastEvent, ToastLabel};
use de_lobby_client::{ListGamesRequest, RequestEvent, ResponseEvent};
use de_lobby_model::GamePartial;
use iyes_loopless::prelude::*;

use crate::{menu::Menu, MenuState};

const REFRESH_INTERVAL: Duration = Duration::from_secs(10);

pub(crate) struct GameListingPlugin;

impl Plugin for GameListingPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(MenuState::GameListing, setup)
            .add_exit_system(MenuState::GameListing, cleanup)
            .add_system(refresh_system.run_in_state(MenuState::GameListing))
            .add_system(
                list_games_system
                    .run_in_state(MenuState::GameListing)
                    .before(ToastLabel::ProcessEvents),
            )
            .add_system(button_system.run_in_state(MenuState::GameListing));
    }
}

#[derive(Resource)]
struct GamesTable(Entity);

#[derive(Component)]
enum ButtonAction {
    Create,
    Join,
}

fn setup(
    mut commands: GuiCommands,
    menu: Res<Menu>,
    mut requests: EventWriter<RequestEvent<ListGamesRequest>>,
) {
    let column_id = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                size: Size::new(Val::Percent(80.), Val::Percent(80.)),
                margin: UiRect::all(Val::Auto),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            ..default()
        })
        .id();
    commands.entity(menu.root_node()).add_child(column_id);

    create_game_button(&mut commands, column_id);
    let table_id = table(&mut commands, column_id);
    commands.insert_resource(GamesTable(table_id));
    requests.send(RequestEvent::new("list-games", ListGamesRequest));
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<GamesTable>();
}

fn create_game_button(commands: &mut GuiCommands, parent_node: Entity) {
    let button_id = commands
        .spawn_button(
            OuterStyle {
                size: Size::new(Val::Percent(100.), Val::Percent(8.)),
                margin: UiRect::bottom(Val::Percent(1.)),
            },
            "Create Game",
        )
        .insert(ButtonAction::Create)
        .id();
    commands.entity(parent_node).add_child(button_id);
}

fn table(commands: &mut GuiCommands, parent_node: Entity) -> Entity {
    let table_id = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                size: Size::new(Val::Percent(100.), Val::Percent(91.)),
                margin: UiRect::all(Val::Auto),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            ..default()
        })
        .id();
    commands.entity(parent_node).add_child(table_id);
    table_id
}

fn row(commands: &mut GuiCommands, game: &GamePartial) -> Entity {
    let row_id = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                size: Size::new(Val::Percent(100.), Val::Percent(8.)),
                margin: UiRect::vertical(Val::Percent(0.5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            ..default()
        })
        .id();

    let name_id = commands
        .spawn_label(
            OuterStyle {
                size: Size::new(Val::Percent(80.), Val::Percent(100.)),
                margin: UiRect::right(Val::Percent(2.)),
            },
            format!(
                "{} ({}/{})",
                game.config().name(),
                game.num_players(),
                game.config().max_players()
            ),
        )
        .id();
    commands.entity(row_id).add_child(name_id);

    if game.num_players() < game.config().max_players() {
        let button_id = commands
            .spawn_button(
                OuterStyle {
                    size: Size::new(Val::Percent(18.), Val::Percent(100.)),
                    ..default()
                },
                "Join",
            )
            .insert(ButtonAction::Join)
            .id();
        commands.entity(row_id).add_child(button_id);
    }

    row_id
}

fn refresh_system(
    time: Res<Time>,
    mut stopwatch: Local<Stopwatch>,
    mut requests: EventWriter<RequestEvent<ListGamesRequest>>,
) {
    stopwatch.tick(time.delta());
    if stopwatch.elapsed() >= REFRESH_INTERVAL {
        stopwatch.reset();
        requests.send(RequestEvent::new("list-games", ListGamesRequest));
    }
}

fn list_games_system(
    mut commands: GuiCommands,
    table: Res<GamesTable>,
    mut events: EventReader<ResponseEvent<ListGamesRequest>>,
    mut toasts: EventWriter<ToastEvent>,
) {
    let Some(event) = events.iter().last() else { return };
    commands.entity(table.0).despawn_descendants();

    match event.result() {
        Ok(games) => {
            for game in games.games() {
                let row_id = row(&mut commands, game);
                commands.entity(table.0).add_child(row_id);
            }
        }
        Err(error) => toasts.send(ToastEvent::new(error)),
    }
}

fn button_system(
    mut commands: Commands,
    interactions: Query<(&Interaction, &ButtonAction), Changed<Interaction>>,
    mut toasts: EventWriter<ToastEvent>,
) {
    for (&interaction, action) in interactions.iter() {
        if let Interaction::Clicked = interaction {
            match action {
                ButtonAction::Create => {
                    commands.insert_resource(NextState(MenuState::GameCreation))
                }
                ButtonAction::Join => {
                    toasts.send(ToastEvent::new("Not yet implemented (issue #301)."))
                }
            }
        }
    }
}
