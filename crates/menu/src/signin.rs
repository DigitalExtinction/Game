use bevy::prelude::*;
use de_gui::{
    ButtonCommands, GuiCommands, LabelCommands, OuterStyle, SetFocusEvent, TextBoxCommands,
    TextBoxQuery, ToastEvent,
};
use de_lobby_client::{Authentication, LobbyRequest, SignInRequest, SignUpRequest};
use de_lobby_model::{User, UserWithPassword, UsernameAndPassword};

use crate::{
    menu::Menu,
    requests::{Receiver, RequestsPlugin, Sender},
    MenuState,
};

pub(crate) struct SignInPlugin;

impl Plugin for SignInPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RequestsPlugin::<SignInRequest>::new(),
            RequestsPlugin::<SignUpRequest>::new(),
        ))
        .add_systems(OnEnter(MenuState::SignIn), setup)
        .add_systems(OnExit(MenuState::SignIn), cleanup)
        .add_systems(
            Update,
            (
                button_system.run_if(resource_exists::<Inputs>()),
                response_system::<SignInRequest>,
                response_system::<SignUpRequest>,
                auth_system,
            )
                .run_if(in_state(MenuState::SignIn)),
        );
    }
}

#[derive(Resource)]
struct Inputs {
    username: Entity,
    password: Entity,
}

#[derive(Component, Clone, Copy)]
enum Action {
    SignIn,
    SignUp,
}

fn setup(mut commands: GuiCommands, menu: Res<Menu>, mut focus: EventWriter<SetFocusEvent>) {
    let column = root_column(&mut commands);
    commands.entity(menu.root_node()).add_child(column);

    let username_row = row(&mut commands, column);
    let input_text_box = input(&mut commands, username_row, "Username:", false);
    focus.send(SetFocusEvent::some(input_text_box));

    let password_row = row(&mut commands, column);
    let password_text_box = input(&mut commands, password_row, "Password:", true);

    let buttons_row = row(&mut commands, column);
    buttons(&mut commands, buttons_row);

    commands.insert_resource(Inputs {
        username: input_text_box,
        password: password_text_box,
    });
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<Inputs>();
}

fn root_column(commands: &mut GuiCommands) -> Entity {
    commands
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
        .id()
}

fn row(commands: &mut GuiCommands, parent: Entity) -> Entity {
    let id = commands
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
    commands.entity(parent).add_child(id);
    id
}

fn input(commands: &mut GuiCommands, parent: Entity, caption: &str, secret: bool) -> Entity {
    let caption = commands
        .spawn_label(
            OuterStyle {
                width: Val::Percent(35.),
                height: Val::Percent(100.),
                ..default()
            },
            caption,
        )
        .id();
    commands.entity(parent).add_child(caption);

    let input = commands
        .spawn_text_box(
            OuterStyle {
                width: Val::Percent(65.),
                height: Val::Percent(100.),
                ..default()
            },
            secret,
        )
        .id();
    commands.entity(parent).add_child(input);
    input
}

fn buttons(commands: &mut GuiCommands, parent: Entity) {
    button(commands, parent, Action::SignIn);
    button(commands, parent, Action::SignUp);
}

fn button(commands: &mut GuiCommands, parent: Entity, action: Action) {
    let caption = match action {
        Action::SignIn => "Sign In",
        Action::SignUp => "Sign Up",
    };

    let id = commands
        .spawn_button(
            OuterStyle {
                width: Val::Percent(48.),
                height: Val::Percent(100.),
                ..default()
            },
            caption,
        )
        .insert(action)
        .id();
    commands.entity(parent).add_child(id);
}

fn button_system(
    inputs: Res<Inputs>,
    texts: TextBoxQuery,
    interactions: Query<(&Interaction, &Action), Changed<Interaction>>,
    mut sign_in_sender: Sender<SignInRequest>,
    mut sign_up_sender: Sender<SignUpRequest>,
) {
    for (&interaction, &action) in interactions.iter() {
        if let Interaction::Pressed = interaction {
            let username = texts.text(inputs.username).unwrap().to_string();
            let password = texts.text(inputs.password).unwrap().to_string();

            match action {
                Action::SignIn => {
                    sign_in_sender.send(SignInRequest::new(UsernameAndPassword::new(
                        username, password,
                    )));
                }
                Action::SignUp => {
                    sign_up_sender.send(SignUpRequest::new(UserWithPassword::new(
                        password,
                        User::new(username),
                    )));
                }
            }

            break;
        }
    }
}

fn response_system<T>(mut receiver: Receiver<T>, mut toasts: EventWriter<ToastEvent>)
where
    T: LobbyRequest,
{
    if let Some(Err(error)) = receiver.receive() {
        toasts.send(ToastEvent::new(error));
    }
}

fn auth_system(mut next_state: ResMut<NextState<MenuState>>, auth: Res<Authentication>) {
    if auth.is_authenticated() {
        next_state.set(MenuState::GameListing);
    }
}
