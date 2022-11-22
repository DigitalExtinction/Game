//! Object spawning and drafting functionalities.

use bevy::{app::PluginGroupBuilder, prelude::*};
use counter::CounterPlugin;
pub use counter::ObjectCounter;
use destroyer::DestroyerPlugin;
use draft::DraftPlugin;
pub use draft::{Draft, DraftBundle};
pub use spawner::SpawnBundle;
use spawner::SpawnerPlugin;

mod counter;
mod destroyer;
mod draft;
mod spawner;

pub struct SpawnerPluginGroup;

impl PluginGroup for SpawnerPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CounterPlugin)
            .add(SpawnerPlugin)
            .add(DraftPlugin)
            .add(DestroyerPlugin)
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub enum SpawnerLabels {
    Destroyer,
}
