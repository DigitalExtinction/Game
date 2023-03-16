use bevy::{app::PluginGroupBuilder, prelude::*};
pub use manufacturing::EnqueueAssemblyEvent;
use manufacturing::ManufacturingPlugin;

mod manufacturing;

pub struct ConstructionPluginGroup;

impl PluginGroup for ConstructionPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(ManufacturingPlugin)
    }
}
