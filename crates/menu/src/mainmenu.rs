use bevy::prelude::*;
use de_core::{
    gconfig::GameConfig,
    player::Player,
    state::{AppState, GameState},
};
use iyes_loopless::prelude::*;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);

pub(crate) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::InMenu, setup)
            .add_exit_system(AppState::InMenu, cleanup)
            .add_system(button_system);
    }
}

type Interactions<'w, 'q> = Query<
    'w,
    'q,
    (&'static Interaction, &'static mut UiColor),
    (Changed<Interaction>, With<Button>),
>;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Percent(25.), Val::Percent(10.)),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Start Game",
                TextStyle {
                    font: asset_server.load("fonts/Fira_Mono/FiraMono-Medium.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        });
}

fn cleanup(
    mut commands: Commands,
    camera: Query<Entity, With<Camera2d>>,
    nodes: Query<Entity, (With<Node>, Without<Parent>)>,
) {
    for entity in camera.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in nodes.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn button_system(mut commands: Commands, mut interactions: Interactions) {
    for (&interaction, mut color) in interactions.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                commands.insert_resource(GameConfig::new("map.tar", Player::Player1));
                commands.insert_resource(NextState(AppState::InGame));
                commands.insert_resource(NextState(GameState::Loading));
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
