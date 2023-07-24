use bevy::prelude::*;
use input::InputPlugin;
pub(crate) use input::{
    DragUpdateType, MouseClickedEvent, MouseDoubleClickedEvent, MouseDraggedEvent, MousePosition,
    MouseSet,
};
use pointer::PointerPlugin;
pub(crate) use pointer::{Pointer, PointerSet};

mod input;
mod pointer;

pub(crate) struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputPlugin).add_plugin(PointerPlugin);
    }
}
