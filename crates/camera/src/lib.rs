use bevy::{app::PluginGroupBuilder, prelude::*};
use camera::CameraPlugin;
pub use camera::{
    CameraLabel, MoveCameraHorizontallyEvent, MoveFocusEvent, RotateCameraEvent, TiltCameraEvent,
    ZoomCameraEvent,
};
use distance::DistancePlugin;
pub use distance::{CameraDistance, DistanceLabels};

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
