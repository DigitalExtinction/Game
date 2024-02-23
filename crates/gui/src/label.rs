use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::{GuiCommands, OuterStyle};

pub trait LabelCommands<'w, 's> {
    fn spawn_label(&mut self, size: OuterStyle, caption: impl Into<String>) -> EntityCommands<'_>;
}

impl<'w, 's> LabelCommands<'w, 's> for GuiCommands<'w, 's> {
    fn spawn_label(&mut self, style: OuterStyle, caption: impl Into<String>) -> EntityCommands<'_> {
        let text_style = self.text_props().label_text_style();

        let mut commands = self.spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                width: style.width,
                height: style.height,
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
