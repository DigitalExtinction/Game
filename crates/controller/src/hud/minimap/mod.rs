use bevy::prelude::*;

use self::{fill::FillPlugin, interaction::InteractionPlugin, nodes::NodesPlugin};

mod draw;
mod fill;
mod interaction;
mod nodes;

pub(crate) struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(NodesPlugin)
            .add_plugin(FillPlugin)
            .add_plugin(InteractionPlugin);
    }
}
