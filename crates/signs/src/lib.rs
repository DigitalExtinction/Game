use bars::BarsPlugin;
pub use bars::{UpdateBarValueEvent, UpdateBarVisibilityEvent};
use bevy::{app::PluginGroupBuilder, prelude::*};
use markers::MarkersPlugin;

mod bars;
mod markers;

/// The 3D signs are not displayed if further than this from the camera.
const MAX_VISIBILITY_DISTANCE: f32 = 140.;
const DISTANCE_FLAG_BIT: u32 = 0;

pub struct SignsPluginGroup;

impl PluginGroup for SignsPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(BarsPlugin).add(MarkersPlugin);
    }
}
