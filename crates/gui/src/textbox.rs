use std::{borrow::Cow, iter::repeat};

use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    window::PrimaryWindow,
};
use de_core::schedule::InputSchedule;

use crate::{focus::FocusedQuery, GuiCommands, OuterStyle};

const FOCUSED_COLOR: Color = Color::WHITE;
const INACTIVE_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

pub(crate) struct TextBoxPlugin;

impl Plugin for TextBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            InputSchedule,
            (
                focus_system,
                input_system
                    .run_if(on_event::<ReceivedCharacter>().or_else(on_event::<KeyboardInput>())),
            ),
        );
    }
}

pub trait TextBoxCommands<'w, 's> {
    fn spawn_text_box<'a>(
        &'a mut self,
        size: OuterStyle,
        secret: bool,
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> TextBoxCommands<'w, 's> for GuiCommands<'w, 's> {
    fn spawn_text_box<'a>(
        &'a mut self,
        style: OuterStyle,
        secret: bool,
    ) -> EntityCommands<'w, 's, 'a> {
        let text_style = self.text_props().input_text_style();

        let mut commands = self.spawn(NodeBundle {
            style: Style {
                padding: UiRect::horizontal(Val::Percent(2.)),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                width: style.width,
                height: style.height,
                margin: style.margin,
                ..default()
            },
            background_color: INACTIVE_COLOR.into(),
            ..default()
        });

        commands
            .insert(Interaction::None)
            .insert(TextBox::new(secret))
            .with_children(|builder| {
                builder.spawn(
                    TextBundle::from_section("", text_style)
                        .with_text_alignment(TextAlignment::Left),
                );
            });

        commands
    }
}

#[derive(SystemParam)]
pub struct TextBoxQuery<'w, 's> {
    query: Query<'w, 's, &'static TextBox>,
}

impl<'w, 's> TextBoxQuery<'w, 's> {
    pub fn text(&self, entity: Entity) -> Option<Cow<'_, str>> {
        self.query.get(entity).map(|e| e.text()).ok()
    }
}

#[derive(Component)]
pub struct TextBox {
    text: String,
    secret: bool,
}

impl TextBox {
    fn new(secret: bool) -> Self {
        Self {
            text: String::new(),
            secret,
        }
    }

    fn text(&self) -> Cow<'_, str> {
        Cow::from(&self.text)
    }

    fn ui_text(&self) -> String {
        if self.secret {
            String::from_iter(repeat('\u{25CF}').take(self.text.len()))
        } else {
            self.text.clone()
        }
    }

    fn push(&mut self, input: char) {
        debug_assert!(!input.is_control());
        self.text.push(input);
    }

    fn backspace(&mut self) {
        self.text.pop();
    }
}

fn focus_system(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut focused: FocusedQuery<&mut BackgroundColor, With<TextBox>>,
) {
    if focused.is_changed() {
        if let Some(mut color) = focused.get_previous_mut() {
            *color = INACTIVE_COLOR.into();
        }

        let mut window = window_query.single_mut();
        match focused.get_current_mut() {
            Some(mut color) => {
                *color = FOCUSED_COLOR.into();
                window.ime_enabled = true;
            }
            None => window.ime_enabled = false,
        }
    }
}

fn input_system(
    mut focused: FocusedQuery<(&mut TextBox, &Children)>,
    mut texts: Query<&mut Text>,
    mut characters: EventReader<ReceivedCharacter>,
    mut keyboard: EventReader<KeyboardInput>,
) {
    let Some((mut text_box, children)) = focused.get_current_mut() else {
        return;
    };

    let text_id = children
        .iter()
        .cloned()
        .find(|&e| texts.contains(e))
        .expect("Text box without `Text` child component.");
    let mut text = texts.get_mut(text_id).unwrap();

    // FIXME: Fix ordering of multiple event streams once
    // https://github.com/bevyengine/bevy/issues/5984 is fixed.
    for event in characters.iter() {
        if !event.char.is_control() {
            text_box.push(event.char);
        }
    }

    for event in keyboard.iter() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match event.key_code {
            Some(KeyCode::Back) => text_box.backspace(),
            _ => continue,
        }
    }

    text.sections[0].value = text_box.ui_text();
}
