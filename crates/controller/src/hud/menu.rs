use core::fmt;

use bevy::prelude::*;
use de_core::{gamestate::GameState, schedule::InputSchedule, state::AppState};
use de_gui::{ButtonCommands, GuiCommands, OuterStyle};

use super::interaction::InteractionBlocker;

pub(crate) struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ToggleGameMenuEvent>()
            .add_systems(OnEnter(GameState::Playing), setup)
            .add_systems(OnExit(GameState::Playing), cleanup)
            .add_systems(
                InputSchedule,
                (
                    toggle_system
                        .run_if(in_state(GameState::Playing))
                        .in_set(GameMenuSet::Toggle),
                    button_system.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub(crate) enum GameMenuSet {
    Toggle,
}

#[derive(Event)]
pub(crate) struct ToggleGameMenuEvent;

#[derive(Component)]
struct PopUpMenu;

#[derive(Component, Clone, Copy)]
enum ButtonAction {
    Quit,
}

impl fmt::Display for ButtonAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Quit => write!(f, "Quit Game"),
        }
    }
}

fn setup(mut commands: GuiCommands) {
    let root_node = commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                left: Val::Percent(0.),
                right: Val::Percent(0.),
                top: Val::Percent(0.),
                bottom: Val::Percent(0.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            visibility: Visibility::Hidden,
            z_index: ZIndex::Local(1000),
            ..default()
        })
        .insert((PopUpMenu, InteractionBlocker))
        .id();

    let menu_node = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(25.),
                height: Val::Percent(50.),
                padding: UiRect::horizontal(Val::Percent(1.)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::BLACK.into(),
            ..default()
        })
        .id();
    commands.entity(root_node).add_child(menu_node);

    button(&mut commands, menu_node, ButtonAction::Quit);
}

fn button(commands: &mut GuiCommands, parent: Entity, action: ButtonAction) {
    let button = commands
        .spawn_button(
            OuterStyle {
                width: Val::Percent(100.),
                height: Val::Percent(16.),
                margin: UiRect::new(
                    Val::Percent(0.),
                    Val::Percent(0.),
                    Val::Percent(2.),
                    Val::Percent(2.),
                ),
            },
            format!("{action}"),
        )
        .insert(action)
        .id();
    commands.entity(parent).add_child(button);
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<PopUpMenu>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn toggle_system(
    mut events: EventReader<ToggleGameMenuEvent>,
    mut query: Query<&mut Visibility, With<PopUpMenu>>,
) {
    if events.iter().count() % 2 == 0 {
        return;
    }

    *query.single_mut() = if query.single() == Visibility::Hidden {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
}

fn button_system(
    mut next_state: ResMut<NextState<AppState>>,
    interactions: Query<(&Interaction, &ButtonAction), Changed<Interaction>>,
) {
    for (&interaction, &action) in interactions.iter() {
        if let Interaction::Pressed = interaction {
            match action {
                ButtonAction::Quit => next_state.set(AppState::InMenu),
            }
        }
    }
}
