use std::{hash::Hash, path::PathBuf};

use ahash::AHashMap;
use bevy::{
    asset::{Asset, AssetPath, RecursiveDependencyLoadState},
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
                let mut asset_path = AssetPath::from(model_path);
                if let Some(label) = Self::label() {
                    asset_path = asset_path.with_label(label);
                }
                let handle = server.load(asset_path);
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
            .map(
                |key| match server.get_recursive_dependency_load_state(self.get(key)) {
                    Some(load_state) => match load_state {
                        RecursiveDependencyLoadState::Failed => panic!("Model loading failed"),
                        RecursiveDependencyLoadState::NotLoaded => false.into(),
                        RecursiveDependencyLoadState::Loading => false.into(),
                        RecursiveDependencyLoadState::Loaded => true.into(),
                    },
                    // TODO is this correct?
                    None => false.into(),
                },
            )
            .reduce(|a, b| a + b)
            .unwrap_or(Progress { done: 0, total: 0 })
    }
}
