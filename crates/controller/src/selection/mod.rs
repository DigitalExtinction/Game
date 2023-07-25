use area::AreaPlugin;
pub(crate) use area::{AreaSelectSet, SelectInRectEvent};
use bevy::prelude::*;
use bookkeeping::BookkeepingPlugin;
pub(crate) use bookkeeping::{SelectEvent, Selected, SelectionMode, SelectionSet};

mod area;
mod bookkeeping;

pub(crate) struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((BookkeepingPlugin, AreaPlugin));
    }
}
