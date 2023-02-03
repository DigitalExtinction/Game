use bevy::prelude::*;

use self::{fill::FillPlugin, nodes::NodesPlugin};

mod draw;
mod fill;
mod nodes;

pub(crate) struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(NodesPlugin).add_plugin(FillPlugin);
    }
}

#[derive(Resource)]
struct MapImageHandle(Handle<Image>);

impl From<Handle<Image>> for MapImageHandle {
    fn from(handle: Handle<Image>) -> Self {
        Self(handle)
    }
}
