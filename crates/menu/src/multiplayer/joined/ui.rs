use bevy::prelude::*;
use de_gui::{ButtonCommands, GuiCommands, LabelCommands, OuterStyle};
use de_lobby_model::GamePlayer;
use de_messages::Readiness;
use de_multiplayer::SetReadinessEvent;

use crate::{menu::Menu, multiplayer::MultiplayerState};

pub(super) struct JoinedGameUiPlugin;

impl Plugin for JoinedGameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RefreshPlayersEvent>()
            .add_systems(OnEnter(MultiplayerState::GameJoined), setup)
            .add_systems(OnExit(MultiplayerState::GameJoined), cleanup)
            .add_systems(
                Update,
                button_system.run_if(in_state(MultiplayerState::GameJoined)),
            )
            .add_systems(
                PostUpdate,
                refresh
                    .run_if(in_state(MultiplayerState::GameJoined))
                    .run_if(on_event::<RefreshPlayersEvent>()),
            );
    }
}

#[derive(Event)]
pub(super) struct RefreshPlayersEvent(Vec<GamePlayer>);

impl RefreshPlayersEvent {
    pub(super) fn from_slice(players: &[GamePlayer]) -> Self {
        Self(players.to_vec())
    }
}

#[derive(Resource)]
struct PlayersBoxRes(Entity);

#[derive(Clone, Copy, Component)]
enum ButtonAction {
    Ready,
}

fn setup(mut commands: GuiCommands, menu: Res<Menu>) {
    let mid_panel_id = mid_panel(&mut commands, menu.root_node());
    let players_box_id = players_box(&mut commands, mid_panel_id);
    commands.insert_resource(PlayersBoxRes(players_box_id));
    ready_button(&mut commands, mid_panel_id);
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<PlayersBoxRes>();
}

fn mid_panel(commands: &mut GuiCommands, parent_id: Entity) -> Entity {
    let panel_id = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(80.),
                height: Val::Percent(80.),
                margin: UiRect::all(Val::Auto),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            ..default()
        })
        .id();
    commands.entity(parent_id).add_child(panel_id);
    panel_id
}

fn players_box(commands: &mut GuiCommands, parent_id: Entity) -> Entity {
    let column_id = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.),
                height: Val::Percent(80.),
                margin: UiRect::all(Val::Auto),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            ..default()
        })
        .id();
    commands.entity(parent_id).add_child(column_id);
    column_id
}

fn refresh(
    mut commands: GuiCommands,
    mut events: EventReader<RefreshPlayersEvent>,
    box_id: Res<PlayersBoxRes>,
) {
    let Some(event) = events.read().last() else {
        return;
    };

    commands.entity(box_id.0).despawn_descendants();

    for player in event.0.iter() {
        let row_id = row(&mut commands, player);
        commands.entity(box_id.0).add_child(row_id);
    }
}

fn row(commands: &mut GuiCommands, player: &GamePlayer) -> Entity {
    let row_id = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                width: Val::Percent(100.),
                height: Val::Percent(8.),
                margin: UiRect::vertical(Val::Percent(0.5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            ..default()
        })
        .id();

    let ordinal_id = commands
        .spawn_label(
            OuterStyle {
                width: Val::Percent(25.),
                height: Val::Percent(100.),
                margin: UiRect::right(Val::Percent(5.)),
            },
            format!("P{}", player.info().ordinal()),
        )
        .id();
    commands.entity(row_id).add_child(ordinal_id);

    let username_id = commands
        .spawn_label(
            OuterStyle {
                width: Val::Percent(70.),
                height: Val::Percent(100.),
                ..default()
            },
            player.username(),
        )
        .id();
    commands.entity(row_id).add_child(username_id);

    row_id
}

fn ready_button(commands: &mut GuiCommands, parent: Entity) {
    let button_id = commands
        .spawn_button(
            OuterStyle {
                width: Val::Percent(100.),
                height: Val::Percent(8.),
                margin: UiRect::top(Val::Percent(12.)),
            },
            "Ready",
        )
        .insert(ButtonAction::Ready)
        .id();
    commands.entity(parent).add_child(button_id);
}

fn button_system(
    interactions: Query<(&Interaction, &ButtonAction), Changed<Interaction>>,
    mut events: EventWriter<SetReadinessEvent>,
) {
    for (&interaction, &action) in interactions.iter() {
        if let Interaction::Pressed = interaction {
            match action {
                ButtonAction::Ready => {
                    events.send(SetReadinessEvent::from(Readiness::Ready));
                }
            }
        }
    }
}
