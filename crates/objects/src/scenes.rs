use ahash::AHashMap;
use bevy::prelude::*;
use de_core::{objects::ObjectType, state::AppState};
use enum_iterator::Sequence;
use iyes_progress::prelude::*;

use crate::{
    collection::{AssetCollection, AssetCollectionLoader},
    names::FileStem,
};

pub(crate) struct ScenesPlugin;

impl Plugin for ScenesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup.in_schedule(OnEnter(AppState::AppLoading)))
            .add_system(
                check_status
                    .track_progress()
                    .run_if(in_state(AppState::AppLoading)),
            );
    }
}

#[derive(Resource)]
pub struct Scenes(AHashMap<SceneType, Handle<Scene>>);

impl AssetCollection for Scenes {
    type Key = SceneType;
    type Asset = Scene;

    fn get(&self, key: Self::Key) -> &Handle<Self::Asset> {
        self.0.get(&key).unwrap()
    }
}

impl AssetCollectionLoader for Scenes {
    const DIRECTORY: &'static str = "models";
    const SUFFIX: &'static str = "glb";

    fn new(map: AHashMap<Self::Key, Handle<Self::Asset>>) -> Self {
        Self(map)
    }

    fn label() -> Option<String> {
        Some("Scene0".to_owned())
    }
}

#[derive(Sequence, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneType {
    Solid(ObjectType),
    Pole,
}

impl FileStem for SceneType {
    fn stem(self) -> &'static str {
        match self {
            Self::Solid(object_type) => object_type.stem(),
            Self::Pole => "pole",
        }
    }
}

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(Scenes::init(server.as_ref()));
}

fn check_status(server: Res<AssetServer>, scenes: Res<Scenes>) -> Progress {
    scenes.progress(server.as_ref())
}
