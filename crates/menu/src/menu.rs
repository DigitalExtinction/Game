use bevy::prelude::*;
use de_core::state::AppState;
use de_gui::{ButtonCommands, GuiCommands, OuterStyle};

use crate::MenuState;

pub(crate) struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(AppState::InMenu)))
            .add_system(cleanup.in_schedule(OnExit(AppState::InMenu)))
            .add_system(
                clean_up_root
                    .in_base_set(CoreSet::PreUpdate)
                    .run_if(resource_exists::<Menu>()),
            )
            .add_system(
                hide_show_corner
                    .run_if(resource_exists::<Menu>())
                    .run_if(resource_changed::<State<MenuState>>()),
            )
            .add_system(button_system.run_if(in_state(AppState::InMenu)));
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
    *corner_visibility = if state.0 == MenuState::MainMenu {
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
                position: UiRect::all(Val::Percent(0.)),
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                align_items: AlignItems::Center,
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
                position: UiRect::new(
                    Val::Percent(90.),
                    Val::Percent(5.),
                    Val::Percent(5.),
                    Val::Percent(90.),
                ),
                ..default()
            },
            z_index: ZIndex::Global(1),
            ..default()
        })
        .id();

    let close_button = commands
        .spawn_button(
            OuterStyle {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
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
        if let Interaction::Clicked = interaction {
            match action {
                ButtonAction::Close => next_state.set(MenuState::MainMenu),
            }
        }
    }
}
