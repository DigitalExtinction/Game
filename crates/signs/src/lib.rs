use bars::BarsPlugin;
pub use bars::{UpdateBarValueEvent, UpdateBarVisibilityEvent};
use bevy::{app::PluginGroupBuilder, prelude::*};

mod bars;

pub struct SignsPluginGroup;

impl PluginGroup for SignsPluginGroup {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(BarsPlugin);
    }
}
