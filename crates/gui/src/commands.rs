use std::ops::{Deref, DerefMut};

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::text::TextProps;

#[derive(SystemParam)]
pub struct GuiCommands<'w, 's> {
    commands: Commands<'w, 's>,
    text_props: Res<'w, TextProps>,
}

impl<'w, 's> GuiCommands<'w, 's> {
    pub(crate) fn text_props(&self) -> &TextProps {
        self.text_props.as_ref()
    }
}

impl<'w, 's> Deref for GuiCommands<'w, 's> {
    type Target = Commands<'w, 's>;

    fn deref(&self) -> &Self::Target {
        &self.commands
    }
}

impl<'w, 's> DerefMut for GuiCommands<'w, 's> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.commands
    }
}
