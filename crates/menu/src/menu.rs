use bevy::prelude::*;
use de_core::state::{AppState, MenuState};
use iyes_loopless::{prelude::*, state::StateTransitionStageLabel};

pub(crate) struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_before(
            StateTransitionStageLabel::from_type::<MenuState>(),
            MenuStage::PreTransition,
            SystemStage::parallel(),
        )
        .add_enter_system(AppState::InMenu, setup)
        .add_exit_system(AppState::InMenu, cleanup)
        .add_system_to_stage(
            MenuStage::PreTransition,
            state_transition.run_if_resource_exists::<Menu>(),
        );
    }
}

#[derive(StageLabel)]
pub enum MenuStage {
    PreTransition,
}

#[derive(Resource)]
pub(crate) struct Menu {
    root_node: Entity,
}

impl Menu {
    fn new(root_node: Entity) -> Self {
        Self { root_node }
    }

    pub(crate) fn root_node(&self) -> Entity {
        self.root_node
    }
}

fn state_transition(
    mut commands: Commands,
    state: Option<Res<NextState<MenuState>>>,
    menu: Res<Menu>,
) {
    if state.is_some() {
        commands.entity(menu.root_node()).despawn_descendants();
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    let root_node = commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect::all(Val::Percent(0.)),
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                ..default()
            },
            background_color: Color::GRAY.into(),
            ..default()
        })
        .id();
    commands.insert_resource(Menu::new(root_node));
}

fn cleanup(mut commands: Commands, menu: Res<Menu>, camera: Query<Entity, With<Camera2d>>) {
    commands.entity(menu.root_node()).despawn_recursive();
    commands.remove_resource::<Menu>();

    for entity in camera.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
