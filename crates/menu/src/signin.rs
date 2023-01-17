use bevy::prelude::*;
use de_core::state::MenuState;
use de_gui::{
    ButtonCommands, GuiCommands, LabelCommands, OuterStyle, SetFocusEvent, TextBoxCommands,
    TextBoxQuery,
};
use de_lobby_client::{Authentication, RequestEvent, ResponseEvent, SignInRequest, SignUpRequest};
use de_lobby_model::{Token, User, UserWithPassword, UsernameAndPassword};
use iyes_loopless::prelude::*;

use crate::menu::despawn_root_nodes;

pub(crate) struct SignInPlugin;

impl Plugin for SignInPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Counter>()
            .add_enter_system(MenuState::SignIn, setup)
            .add_exit_system(MenuState::SignIn, despawn_root_nodes)
            .add_exit_system(MenuState::SignIn, cleanup)
            .add_system(
                button_system
                    .run_if_resource_exists::<Inputs>()
                    .run_in_state(MenuState::SignIn),
            )
            .add_system(response_system.run_in_state(MenuState::SignIn))
            .add_system(auth_system.run_in_state(MenuState::SignIn));
    }
}

#[derive(Resource, Default)]
pub(crate) struct Counter {
    counter: u64,
}

impl Counter {
    fn increment(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        self.counter
    }

    fn compare(&self, id: &str) -> bool {
        self.counter.to_string() == id
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

fn setup(mut commands: GuiCommands, mut focus: EventWriter<SetFocusEvent>) {
    let root = root_column(&mut commands);

    let username_row = row(&mut commands, root);
    let input_text_box = input(&mut commands, username_row, "Username:", false);
    focus.send(SetFocusEvent::some(input_text_box));

    let password_row = row(&mut commands, root);
    let password_text_box = input(&mut commands, password_row, "Password:", true);

    let buttons_row = row(&mut commands, root);
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
                size: Size::new(Val::Percent(50.), Val::Percent(100.)),
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
                size: Size::new(Val::Percent(100.), Val::Percent(8.)),
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
                size: Size::new(Val::Percent(35.), Val::Percent(100.)),
                ..default()
            },
            caption,
        )
        .id();
    commands.entity(parent).add_child(caption);

    let input = commands
        .spawn_text_box(
            OuterStyle {
                size: Size::new(Val::Percent(65.), Val::Percent(100.)),
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
                size: Size::new(Val::Percent(48.), Val::Percent(100.)),
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
    mut counter: ResMut<Counter>,
    texts: TextBoxQuery,
    interactions: Query<(&Interaction, &Action), Changed<Interaction>>,
    mut sign_in_requests: EventWriter<RequestEvent<SignInRequest>>,
    mut sign_up_requests: EventWriter<RequestEvent<SignUpRequest>>,
) {
    for (&interaction, &action) in interactions.iter() {
        if let Interaction::Clicked = interaction {
            let username = texts.text(inputs.username).unwrap().to_string();
            let password = texts.text(inputs.password).unwrap().to_string();
            let request_id = counter.increment();

            match action {
                Action::SignIn => {
                    sign_in_requests.send(RequestEvent::new(
                        request_id,
                        SignInRequest::new(UsernameAndPassword::new(username, password)),
                    ));
                }
                Action::SignUp => {
                    sign_up_requests.send(RequestEvent::new(
                        request_id,
                        SignUpRequest::new(UserWithPassword::new(password, User::new(username))),
                    ));
                }
            }

            break;
        }
    }
}

fn response_system(counter: Res<Counter>, mut responses: EventReader<ResponseEvent<Token>>) {
    for event in responses.iter() {
        if counter.compare(event.id()) {
            if let Err(error) = event.result() {
                warn!("Error: {:?}", error);
            }
        }
    }
}

fn auth_system(mut commands: Commands, auth: Res<Authentication>) {
    if auth.is_authenticated() {
        commands.insert_resource(NextState(MenuState::GameListing));
    }
}
