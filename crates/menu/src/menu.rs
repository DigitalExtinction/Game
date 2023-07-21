use bevy::prelude::*;
use de_core::state::AppState;
use de_gui::{ButtonCommands, GuiCommands, OuterStyle};

use crate::MenuState;

pub(crate) struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InMenu), setup)
            .add_systems(OnExit(AppState::InMenu), cleanup)
            .add_systems(PreUpdate, clean_up_root.run_if(resource_exists::<Menu>()))
            .add_systems(
                Update,
                (
                    hide_show_corner
                        .run_if(resource_exists::<Menu>())
                        .run_if(resource_changed::<State<MenuState>>()),
                    button_system.run_if(in_state(AppState::InMenu)),
                ),
            );
    }
}

#[derive(Resource)]
pub(crate) struct Menu {
    root_node: Entity,
    corner_node: Entity,
}

impl Menu {
    fn new(root_node: Entity, corner_node: Entity) -> Self {
        Self {
            root_node,
            corner_node,
        }
    }

    pub(crate) fn root_node(&self) -> Entity {
        self.root_node
    }

    fn corner_node(&self) -> Entity {
        self.corner_node
    }
}

#[derive(Component, Clone, Copy)]
enum ButtonAction {
    Close,
}

fn clean_up_root(mut commands: Commands, state: Res<NextState<MenuState>>, menu: Res<Menu>) {
    if state.0.is_none() {
        return;
    };
    commands.entity(menu.root_node()).despawn_descendants();
}

fn hide_show_corner(
    state: Res<State<MenuState>>,
    menu: Res<Menu>,
    mut visibility: Query<&mut Visibility>,
) {
    let mut corner_visibility = visibility.get_mut(menu.corner_node()).unwrap();
    *corner_visibility = if state.get() == &MenuState::MainMenu {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
}

fn setup(mut commands: GuiCommands) {
    commands.spawn(Camera2dBundle::default());
    let root_node = spawn_root_node(&mut commands);
    let corner_node = spawn_corner_node(&mut commands);
    commands.insert_resource(Menu::new(root_node, corner_node));
}

fn cleanup(mut commands: Commands, menu: Res<Menu>, camera: Query<Entity, With<Camera2d>>) {
    commands.entity(menu.root_node()).despawn_recursive();
    commands.entity(menu.corner_node()).despawn_recursive();
    commands.remove_resource::<Menu>();

    for entity in camera.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_root_node(commands: &mut GuiCommands) -> Entity {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Percent(0.),
                bottom: Val::Percent(0.),
                left: Val::Percent(0.),
                right: Val::Percent(0.),
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            background_color: Color::GRAY.into(),
            ..default()
        })
        .id()
}

fn spawn_corner_node(commands: &mut GuiCommands) -> Entity {
    let corner_node = commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Percent(90.),
                right: Val::Percent(5.),
                top: Val::Percent(5.),
                bottom: Val::Percent(90.),

                ..default()
            },
            z_index: ZIndex::Global(1),
            ..default()
        })
        .id();

    let close_button = commands
        .spawn_button(
            OuterStyle {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            "X",
        )
        .insert(ButtonAction::Close)
        .id();
    commands.entity(corner_node).add_child(close_button);

    corner_node
}

fn button_system(
    mut next_state: ResMut<NextState<MenuState>>,
    interactions: Query<(&Interaction, &ButtonAction), Changed<Interaction>>,
) {
    for (&interaction, &action) in interactions.iter() {
        if let Interaction::Pressed = interaction {
            match action {
                ButtonAction::Close => next_state.set(MenuState::MainMenu),
            }
        }
    }
}
