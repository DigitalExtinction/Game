use bevy::{app::PluginGroupBuilder, prelude::*};
use camera::CameraPlugin;
pub use camera::MoveFocusEvent;

mod camera;

pub struct CameraPluginGroup;

impl PluginGroup for CameraPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(CameraPlugin);
    }
}
