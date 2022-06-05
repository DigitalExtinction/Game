mod systems;

use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use systems::MovementPlugin;

pub struct MovementPluginGroup;

impl PluginGroup for MovementPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(MovementPlugin);
    }
}
