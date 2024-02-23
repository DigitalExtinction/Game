use bevy::{
    ecs::system::{EntityCommands, SystemParam},
    prelude::*,
};

use crate::{GuiCommands, OuterStyle};

/// marker component for UI `BasicText`
#[derive(Component)]
struct BodyText;

pub trait BodyTextCommands<'w, 's> {
    fn spawn_body_text<'a>(
        &'a mut self,
        size: OuterStyle,
        caption: impl Into<String>,
    ) -> EntityCommands<'a>;
}

impl<'w, 's> BodyTextCommands<'w, 's> for GuiCommands<'w, 's> {
    fn spawn_body_text<'a>(
        &'a mut self,
        style: OuterStyle,
        caption: impl Into<String>,
    ) -> EntityCommands<'a> {
        let text_style = self.text_props().body_text_style();

        let mut commands = self.spawn((
            NodeBundle {
                style: Style {
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::FlexStart,
                    width: style.width,
                    height: style.height,
                    margin: style.margin,
                    ..default()
                },
                ..default()
            },
            BodyText,
        ));

        commands.with_children(|builder| {
            builder.spawn(TextBundle::from_section(caption, text_style));
        });

        commands
    }
}

#[derive(SystemParam)]
pub struct BodyTextOps<'w, 's> {
    body_text_query: Query<'w, 's, &'static Children, With<BodyText>>,
    text_query: Query<'w, 's, &'static mut Text>,
}

impl<'w, 's> BodyTextOps<'w, 's> {
    /// This method changes text (e.g. caption) of UI body text.
    pub fn set_text(&mut self, entity: Entity, text: impl Into<String>) -> Result<(), &str> {
        let text = text.into();
        let children = match self.body_text_query.get(entity) {
            Ok(children) => children,
            Err(e) => {
                trace!("BodyText does not exist. {:?}", e);
                return Err("BodyText does not exist.");
            }
        };
        for &child in children.iter() {
            if let Ok(mut text_component) = self.text_query.get_mut(child) {
                if text_component.sections[0].value == text {
                    // avoid unnecessary update
                    return Ok(());
                }
                text_component.sections[0].value = text;
                return Ok(());
            }
        }
        trace!("BodyText does not have a child with Text component.");
        Err("BodyText does not have a child with Text component.")
    }
}
