//! Object spawning and drafting functionalities.

use bevy::{app::PluginGroupBuilder, prelude::*};
use counter::CounterPlugin;
pub use counter::ObjectCounter;
pub use destroyer::DespawnEvent;
pub use destroyer::DespawnEventsPlugin;
pub use destroyer::DespawnedComponentsEvent;
pub use destroyer::DespawnerSet;
use draft::DraftPlugin;
pub use draft::{DraftAllowed, DraftBundle};
use gameend::GameEndPlugin;
pub use spawner::SpawnBundle;
use spawner::SpawnerPlugin;

use crate::destroyer::DespawnerPlugin;

mod counter;
mod destroyer;
mod draft;
mod gameend;
mod spawner;

pub struct SpawnerPluginGroup;

impl PluginGroup for SpawnerPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CounterPlugin)
            .add(SpawnerPlugin)
            .add(DraftPlugin)
            .add(GameEndPlugin)
            .add(DespawnerPlugin)
    }
}
