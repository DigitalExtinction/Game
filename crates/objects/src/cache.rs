use std::path::PathBuf;

use bevy::{
    asset::{AssetPath, LoadState},
    prelude::*,
};
use de_core::{
    objects::{ActiveObjectType, InactiveObjectType, ObjectType},
    state::GameState,
};
use enum_map::{enum_map, EnumMap};
use iyes_loopless::prelude::*;
use iyes_progress::prelude::*;

pub(crate) struct CachePlugin;

impl Plugin for CachePlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Loading, setup).add_system(
            check_status
                .track_progress()
                .run_in_state(GameState::Loading),
        );
    }
}

pub(crate) struct Cache {
    objects: EnumMap<ObjectType, CacheItem>,
}

impl Cache {
    pub(crate) fn get(&self, object_type: ObjectType) -> &CacheItem {
        &self.objects[object_type]
    }

    fn load(server: &AssetServer) -> Self {
        Self {
            objects: enum_map! {
                ObjectType::Active(ActiveObjectType::Base) => CacheItem::from_name(server, "base"),
                ObjectType::Active(ActiveObjectType::PowerHub) => CacheItem::from_name(server, "powerhub"),
                ObjectType::Active(ActiveObjectType::Attacker) => CacheItem::from_name(server, "attacker"),
                ObjectType::Inactive(InactiveObjectType::Tree) => CacheItem::from_name(server, "tree"),
            },
        }
    }

    fn advance(&mut self, server: &AssetServer) -> Progress {
        self.objects
            .values()
            .map(|i| i.advance(server))
            .reduce(|a, b| a + b)
            .unwrap()
    }
}

pub(crate) struct CacheItem {
    scene: Handle<Scene>,
}

impl CacheItem {
    pub(crate) fn scene(&self) -> Handle<Scene> {
        self.scene.clone()
    }

    fn from_name(server: &AssetServer, name: &str) -> Self {
        let mut path = PathBuf::new();
        path.push("models");
        path.push(format!("{}.glb", name));
        Self {
            scene: server.load(AssetPath::new(path, Some("Scene0".to_owned()))),
        }
    }

    fn advance(&self, server: &AssetServer) -> Progress {
        match server.get_load_state(&self.scene) {
            LoadState::Failed => panic!("Cache item loading failed"),
            LoadState::Unloaded => panic!("Cache item is unexpectedly unloaded"),
            LoadState::NotLoaded => false.into(),
            LoadState::Loading => false.into(),
            LoadState::Loaded => true.into(),
        }
    }
}

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(Cache::load(server.as_ref()));
}

fn check_status(mut cache: ResMut<Cache>, server: Res<AssetServer>) -> Progress {
    cache.advance(server.as_ref())
}
