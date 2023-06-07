use bars::BarsPlugin;
pub use bars::{UpdateBarValueEvent, UpdateBarVisibilityEvent};
use bevy::{app::PluginGroupBuilder, prelude::*};
use markers::MarkersPlugin;
use pole::PolePlugin;
pub use pole::{UpdatePoleLocationEvent, UpdatePoleVisibilityEvent};

mod bars;
mod markers;
mod pole;

/// The 3D signs are not displayed if further than this from the camera.
const MAX_VISIBILITY_DISTANCE: f32 = 140.;
const DISTANCE_FLAG_BIT: u32 = 0;
const UPDATE_TIMER_FLAG_BIT: u32 = 2;

pub struct SignsPluginGroup;

impl PluginGroup for SignsPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(BarsPlugin)
            .add(MarkersPlugin)
            .add(PolePlugin)
    }
}
