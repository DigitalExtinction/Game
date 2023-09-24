use bevy::{app::PluginGroupBuilder, prelude::*};
use camera::CameraPlugin;
pub use camera::{
    CameraFocus, CameraSet, MoveCameraHorizontallyEvent, MoveFocusEvent, RotateCameraEvent,
    TiltCameraEvent, ZoomCameraEvent,
};
use distance::DistancePlugin;
pub use distance::{CameraDistance, DistanceSet};
use skybox::SkyboxPlugin;

mod camera;
mod distance;
mod skybox;

pub struct CameraPluginGroup;

impl PluginGroup for CameraPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CameraPlugin)
            .add(DistancePlugin)
            .add(SkyboxPlugin)
    }
}
