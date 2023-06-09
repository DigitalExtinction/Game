use bevy::{app::PluginGroupBuilder, prelude::*};
use manufacturing::ManufacturingPlugin;
pub use manufacturing::{AssemblyLine, ChangeDeliveryLocationEvent, EnqueueAssemblyEvent};

mod manufacturing;

pub struct ConstructionPluginGroup;

impl PluginGroup for ConstructionPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(ManufacturingPlugin)
    }
}
