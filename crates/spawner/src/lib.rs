//! Object spawning and drafting functionalities.

use bevy::{app::PluginGroupBuilder, prelude::*};
use counter::CounterPlugin;
pub use counter::ObjectCounter;
pub use despawner::{DespawnEvent, DespawnEventsPlugin, DespawnedComponentsEvent, DespawnerSet};
use draft::DraftPlugin;
pub use draft::{DraftAllowed, DraftBundle};
use gameend::GameEndPlugin;
pub use spawner::SpawnBundle;
use spawner::SpawnerPlugin;

use crate::despawner::DespawnerPlugin;

mod counter;
mod despawner;
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
