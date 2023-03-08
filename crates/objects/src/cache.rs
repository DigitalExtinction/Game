use std::{ops::Deref, path::PathBuf, sync::Arc};

use bevy::{
    asset::{Asset, AssetPath, LoadState},
    prelude::*,
};
use de_core::{
    gamestate::GameState,
    objects::{ActiveObjectType, BuildingType, InactiveObjectType, ObjectType, UnitType},
    state::AppState,
};
use enum_map::{enum_map, EnumMap};
use iyes_progress::prelude::*;

use crate::{
    ichnography::Ichnography,
    loader::{ObjectInfo, ObjectLoader},
    Flight, LaserCannon, ObjectCollider,
};

pub(crate) struct CachePlugin;

impl Plugin for CachePlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<ObjectInfo>()
            .add_asset_loader(ObjectLoader)
            .add_system(setup.in_schedule(OnEnter(AppState::InGame)))
            .add_system(cleanup.in_schedule(OnExit(AppState::InGame)))
            .add_system(
                check_status
                    .track_progress()
                    .run_if(in_state(GameState::Loading)),
            );
    }
}

#[derive(Clone, Resource)]
pub struct ObjectCache {
    inner: Arc<InnerCache>,
}

impl ObjectCache {
    fn new(inner: InnerCache) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

impl Deref for ObjectCache {
    type Target = InnerCache;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

pub struct InnerCache {
    objects: EnumMap<ObjectType, CacheItem>,
}

impl InnerCache {
    pub fn get(&self, object_type: ObjectType) -> &CacheItem {
        &self.objects[object_type]
    }
}

pub struct CacheItem {
    scene: Handle<Scene>,
    ichnography: Ichnography,
    collider: ObjectCollider,
    cannon: Option<LaserCannon>,
    flight: Option<Flight>,
}

impl CacheItem {
    pub fn scene(&self) -> Handle<Scene> {
        self.scene.clone()
    }

    pub fn cannon(&self) -> Option<&LaserCannon> {
        self.cannon.as_ref()
    }

    /// Flight configuration configuration. It is None for objects which cannot
    /// fly.
    pub fn flight(&self) -> Option<&Flight> {
        self.flight.as_ref()
    }

    pub(crate) fn ichnography(&self) -> &Ichnography {
        &self.ichnography
    }

    pub(crate) fn collider(&self) -> &ObjectCollider {
        &self.collider
    }
}

#[derive(Resource)]
struct CacheLoader {
    objects: EnumMap<ObjectType, ItemLoader>,
}

impl CacheLoader {
    fn load(server: &AssetServer) -> Self {
        Self {
            objects: enum_map! {
                ObjectType::Active(ActiveObjectType::Building(BuildingType::Base))
                    => ItemLoader::from_name(server, "base"),
                ObjectType::Active(ActiveObjectType::Building(BuildingType::PowerHub))
                    => ItemLoader::from_name(server, "powerhub"),
                ObjectType::Active(ActiveObjectType::Unit(UnitType::Attacker))
                    => ItemLoader::from_name(server, "attacker"),
                ObjectType::Inactive(InactiveObjectType::Tree)
                    => ItemLoader::from_name(server, "tree"),
            },
        }
    }

    fn into_cache(self, objects: &Assets<ObjectInfo>) -> InnerCache {
        InnerCache {
            objects: self
                .objects
                .map(|object_type, loader| loader.into_cache_item(object_type, objects)),
        }
    }

    fn advance(&self, server: &AssetServer) -> Progress {
        self.objects
            .values()
            .map(|i| i.advance(server))
            .reduce(|a, b| a + b)
            .unwrap()
    }
}

pub(crate) struct ItemLoader {
    scene: Handle<Scene>,
    object_info: Handle<ObjectInfo>,
}

impl ItemLoader {
    fn from_name(server: &AssetServer, name: &str) -> Self {
        let mut model_path = PathBuf::new();
        model_path.push("models");
        model_path.push(format!("{name}.glb"));

        let mut object_info_path = PathBuf::new();
        object_info_path.push("objects");
        object_info_path.push(format!("{name}.obj.json"));

        Self {
            scene: server.load(AssetPath::new(model_path, Some("Scene0".to_owned()))),
            object_info: server.load(object_info_path),
        }
    }

    /// # Panics
    ///
    /// Panics if the object is wrongly configured.
    fn into_cache_item(self, object_type: ObjectType, objects: &Assets<ObjectInfo>) -> CacheItem {
        let object_info = objects.get(&self.object_info).unwrap();

        if object_info.flight().is_some() {
            assert!(
                matches!(object_type, ObjectType::Active(ActiveObjectType::Unit(_))),
                "Flight info specified for non-movable object {object_type}."
            );
        }

        CacheItem {
            scene: self.scene,
            ichnography: Ichnography::from(object_info.footprint()),
            collider: ObjectCollider::from(object_info.shape()),
            cannon: object_info.cannon().map(LaserCannon::from),
            flight: object_info.flight().map(Flight::from),
        }
    }

    fn advance(&self, server: &AssetServer) -> Progress {
        Self::advance_single(server, &self.scene) + Self::advance_single(server, &self.object_info)
    }

    fn advance_single<T: Asset>(server: &AssetServer, handle: &Handle<T>) -> Progress {
        match server.get_load_state(handle) {
            LoadState::Failed => panic!("Cache item loading failed"),
            LoadState::Unloaded => panic!("Cache item is unexpectedly unloaded"),
            LoadState::NotLoaded => false.into(),
            LoadState::Loading => false.into(),
            LoadState::Loaded => true.into(),
        }
    }
}

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(CacheLoader::load(server.as_ref()));
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<CacheLoader>();
    commands.remove_resource::<ObjectCache>();
}

fn check_status(
    mut commands: Commands,
    mut progress: Local<Progress>,
    // keep it boxed so the memory can be freed (the system stays around forever)
    mut loader: Local<Option<Box<CacheLoader>>>,
    cache: Option<Res<ObjectCache>>,
    server: Res<AssetServer>,
    objects: Res<Assets<ObjectInfo>>,
) -> Progress {
    if cache.is_some() {
        debug_assert!(loader.is_none());
        debug_assert!(progress.done >= progress.total);
    } else if loader.is_none() && cache.is_none() {
        *progress = false.into();
        *loader = Some(Box::new(CacheLoader::load(server.as_ref())));
    } else {
        *progress = loader.as_ref().unwrap().advance(server.as_ref());
        if progress.done >= progress.total {
            let mut ready_loader = None;
            std::mem::swap(&mut ready_loader, &mut loader);
            let inner_cache = ready_loader.unwrap().into_cache(objects.as_ref());
            commands.insert_resource(ObjectCache::new(inner_cache));
        }
    }

    *progress
}
