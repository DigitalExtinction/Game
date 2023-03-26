use std::{hash::Hash, path::PathBuf};

use ahash::AHashMap;
use bevy::{
    asset::{Asset, AssetPath, LoadState},
    prelude::*,
};
use enum_iterator::Sequence;
use iyes_progress::Progress;

use crate::names::FileStem;

pub trait AssetCollection {
    type Key;
    type Asset: Asset;

    fn get(&self, scene_type: Self::Key) -> &Handle<Self::Asset>;
}

pub(crate) trait AssetCollectionLoader
where
    Self: Sized + AssetCollection,
    Self::Key: Eq + Hash + FileStem + Sequence,
{
    const DIRECTORY: &'static str;
    const SUFFIX: &'static str;

    fn new(map: AHashMap<Self::Key, Handle<Self::Asset>>) -> Self;

    /// Return asset label to be passed to the asset server to load all assets
    /// from the collection.
    fn label() -> Option<String>;

    /// Initialize the collection by (starting) loading of all assets of the
    /// collection.
    fn init(server: &AssetServer) -> Self {
        Self::new(AHashMap::from_iter(enum_iterator::all::<Self::Key>().map(
            |key| {
                let mut model_path = PathBuf::new();
                model_path.push(Self::DIRECTORY);
                model_path.push(format!("{}.{}", key.stem(), Self::SUFFIX));
                let handle = server.load(AssetPath::new(model_path, Self::label()));
                (key, handle)
            },
        )))
    }

    /// Returns progress of the loading.
    ///
    /// # Panics
    ///
    /// Panics if loading any of the assets is either failed or unloaded.
    fn progress(&self, server: &AssetServer) -> Progress {
        enum_iterator::all::<Self::Key>()
            .map(|key| match server.get_load_state(self.get(key)) {
                LoadState::Failed => panic!("Model loading failed"),
                LoadState::Unloaded => panic!("Model is unexpectedly unloaded"),
                LoadState::NotLoaded => false.into(),
                LoadState::Loading => false.into(),
                LoadState::Loaded => true.into(),
            })
            .reduce(|a, b| a + b)
            .unwrap_or(Progress { done: 0, total: 0 })
    }
}
