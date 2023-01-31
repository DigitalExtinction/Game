use bevy::prelude::*;
use input::InputPlugin;
pub(crate) use input::{
    DragUpdateType, MouseClicked, MouseDoubleClicked, MouseDragged, MouseLabels, MousePosition,
};
use pointer::PointerPlugin;
pub(crate) use pointer::{Pointer, PointerLabels};

mod input;
mod pointer;

pub(crate) struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputPlugin).add_plugin(PointerPlugin);
    }
}
