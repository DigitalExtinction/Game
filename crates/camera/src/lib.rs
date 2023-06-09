use bevy::{app::PluginGroupBuilder, prelude::*};
use camera::CameraPlugin;
pub use camera::{
    CameraSet, MoveCameraHorizontallyEvent, MoveFocusEvent, RotateCameraEvent, TiltCameraEvent,
    ZoomCameraEvent,
};
use distance::DistancePlugin;
pub use distance::{CameraDistance, DistanceSet};

mod camera;
mod distance;

pub struct CameraPluginGroup;

impl PluginGroup for CameraPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CameraPlugin)
            .add(DistancePlugin)
    }
}
