use ahash::AHashMap;
use anyhow::Context;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext, LoadedAsset},
    ecs::system::SystemParam,
    prelude::*,
    reflect::{TypePath, TypeUuid},
    utils::BoxedFuture,
};
use de_core::state::AppState;
use de_types::objects::ObjectType;
use iyes_progress::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    cannon::{LaserCannon, LaserCannonSerde},
    collection::AssetCollectionLoader,
    collider::{ColliderSerde, ObjectCollider},
    factory::{Factory, FactorySerde},
    flight::{Flight, FlightSerde},
    ichnography::{FootprintSerde, Ichnography},
    AssetCollection,
};

const OBJECT_EXTENSION: [&str; 1] = ["obj.json"];

pub(crate) struct SolidsPlugin;

impl Plugin for SolidsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<SolidObject>()
            .register_asset_loader(SolidObjectLoader)
            .add_systems(OnEnter(AppState::AppLoading), setup)
            .add_systems(
                Update,
                check_status
                    .track_progress()
                    .run_if(in_state(AppState::AppLoading)),
            );
    }
}

#[derive(Resource)]
pub(crate) struct Solids(AHashMap<ObjectType, Handle<SolidObject>>);

impl AssetCollection for Solids {
    type Key = ObjectType;
    type Asset = SolidObject;

    fn get(&self, object_type: ObjectType) -> &Handle<SolidObject> {
        self.0.get(&object_type).unwrap()
    }
}

impl AssetCollectionLoader for Solids {
    const DIRECTORY: &'static str = "objects";

    const SUFFIX: &'static str = OBJECT_EXTENSION[0];

    fn new(map: AHashMap<Self::Key, Handle<Self::Asset>>) -> Self {
        Self(map)
    }

    fn label() -> Option<String> {
        None
    }
}

#[derive(Asset, TypePath)]
pub struct SolidObject {
    ichnography: Ichnography,
    collider: ObjectCollider,
    cannon: Option<LaserCannon>,
    flight: Option<Flight>,
    factory: Option<Factory>,
}

impl SolidObject {
    pub fn cannon(&self) -> Option<&LaserCannon> {
        self.cannon.as_ref()
    }

    /// Flight configuration configuration. It is None for objects which cannot
    /// fly.
    pub fn flight(&self) -> Option<&Flight> {
        self.flight.as_ref()
    }

    /// Returns None if the object has no manufacturing capabilities, otherwise
    /// it returns info about object manufacturing capabilities.
    pub fn factory(&self) -> Option<&Factory> {
        self.factory.as_ref()
    }

    pub fn ichnography(&self) -> &Ichnography {
        &self.ichnography
    }

    pub fn collider(&self) -> &ObjectCollider {
        &self.collider
    }
}

impl TryFrom<SolidObjectSerde> for SolidObject {
    type Error = anyhow::Error;

    fn try_from(solid_serde: SolidObjectSerde) -> Result<Self, Self::Error> {
        Ok(Self {
            ichnography: Ichnography::try_from(solid_serde.footprint)?,
            collider: ObjectCollider::try_from(solid_serde.shape)?,
            cannon: solid_serde.cannon.map(LaserCannon::try_from).transpose()?,
            flight: solid_serde.flight.map(Flight::try_from).transpose()?,
            factory: solid_serde.factory.map(Factory::try_from).transpose()?,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct SolidObjectSerde {
    footprint: FootprintSerde,
    shape: ColliderSerde,
    cannon: Option<LaserCannonSerde>,
    flight: Option<FlightSerde>,
    factory: Option<FactorySerde>,
}

struct SolidObjectLoader;

impl AssetLoader for SolidObjectLoader {
    type Asset = SolidObject;
    type Settings = ();
    type Error = anyhow::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<Self::Asset>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let solid_serde: SolidObjectSerde =
                serde_json::from_slice(&bytes).context("Failed to parse object JSON")?;
            SolidObject::try_from(solid_serde)
        })
    }

    fn extensions(&self) -> &[&str] {
        OBJECT_EXTENSION.as_slice()
    }
}

#[derive(SystemParam)]
pub struct SolidObjects<'w> {
    solids: Res<'w, Solids>,
    assets: Res<'w, Assets<SolidObject>>,
}

impl<'w> SolidObjects<'w> {
    pub fn get(&self, object_type: ObjectType) -> &SolidObject {
        self.assets.get(self.solids.get(object_type)).unwrap()
    }
}

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(Solids::init(server.as_ref()));
}

fn check_status(server: Res<AssetServer>, solids: Res<Solids>) -> Progress {
    solids.progress(server.as_ref())
}
