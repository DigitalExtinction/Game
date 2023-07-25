use bevy::prelude::*;

use self::{fill::FillPlugin, interaction::InteractionPlugin, nodes::NodesPlugin};

mod draw;
mod fill;
mod interaction;
mod nodes;

pub(crate) struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((NodesPlugin, FillPlugin, InteractionPlugin));
    }
}
