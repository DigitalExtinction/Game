use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{GuiCommands, OuterStyle};

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);

pub(crate) struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(color_system);
    }
}

pub trait ButtonCommands<'w, 's> {
    fn spawn_button<'a>(
        &'a mut self,
        size: OuterStyle,
        caption: impl Into<String>,
    ) -> EntityCommands<'w, 's, 'a>;
}

impl<'w, 's> ButtonCommands<'w, 's> for GuiCommands<'w, 's> {
    fn spawn_button<'a>(
        &'a mut self,
        style: OuterStyle,
        caption: impl Into<String>,
    ) -> EntityCommands<'w, 's, 'a> {
        let text_style = self.text_props().button_text_style();

        let mut commands = self.spawn(ButtonBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                size: style.size,
                margin: style.margin,
                ..default()
            },
            ..default()
        });

        commands.with_children(|builder| {
            builder.spawn(TextBundle::from_section(caption, text_style));
        });

        commands
    }
}

type ButtonInteractions<'w, 'q> = Query<
    'w,
    'q,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<Button>),
>;

fn color_system(mut interactions: ButtonInteractions) {
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
