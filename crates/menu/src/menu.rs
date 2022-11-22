use bevy::prelude::*;
use de_core::state::AppState;
use iyes_loopless::prelude::*;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);

pub(crate) struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::InMenu, setup)
            .add_exit_system(AppState::InMenu, cleanup)
            .add_system(button_colors.run_in_state(AppState::InMenu));
    }
}

/// A resource which for handling fonts withing menu.
#[derive(Resource)]
pub(crate) struct Text(Handle<Font>);

impl Text {
    pub(crate) fn button_text_style(&self) -> TextStyle {
        TextStyle {
            font: self.main(),
            font_size: 40.0,
            color: Color::rgb(0.9, 0.9, 0.9),
        }
    }

    fn main(&self) -> Handle<Font> {
        self.0.clone()
    }
}

type ButtonInteractions<'w, 'q> = Query<
    'w,
    'q,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<Button>),
>;

/// This system recursively despawns all `Node`s with no parents.
pub(crate) fn despawn_root_nodes(
    mut commands: Commands,
    nodes: Query<Entity, (With<Node>, Without<Parent>)>,
) {
    for entity in nodes.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let font = asset_server.load("fonts/Fira_Mono/FiraMono-Medium.ttf");
    commands.insert_resource(Text(font));
}

fn cleanup(mut commands: Commands, camera: Query<Entity, With<Camera2d>>) {
    for entity in camera.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn button_colors(mut interactions: ButtonInteractions) {
    for (&interaction, mut color) in interactions.iter_mut() {
        match interaction {
            Interaction::Clicked => (),
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
