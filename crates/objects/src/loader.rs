use std::collections::HashSet;

use anyhow::{Context, Result};
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    reflect::TypeUuid,
};
use de_core::objects::UnitType;
use serde::{Deserialize, Serialize};

const OBJECT_EXTENSION: [&str; 1] = ["obj.json"];

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "5f938388-ebe1-4bb2-bb66-f3e182e4e0bb"]
pub(crate) struct ObjectInfo {
    footprint: Footprint,
    shape: TriMeshShape,
    cannon: Option<LaserCannonInfo>,
    flight: Option<FlightInfo>,
    factory: Option<FactoryInfo>,
}

impl ObjectInfo {
    pub(crate) fn footprint(&self) -> &Footprint {
        &self.footprint
    }

    pub(crate) fn shape(&self) -> &TriMeshShape {
        &self.shape
    }

    pub(crate) fn cannon(&self) -> Option<&LaserCannonInfo> {
        self.cannon.as_ref()
    }

    pub(crate) fn flight(&self) -> Option<&FlightInfo> {
        self.flight.as_ref()
    }

    pub(crate) fn factory(&self) -> Option<&FactoryInfo> {
        self.factory.as_ref()
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Footprint {
    convex_hull: Vec<[f32; 2]>,
}

impl Footprint {
    pub(crate) fn convex_hull(&self) -> &[[f32; 2]] {
        self.convex_hull.as_slice()
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct TriMeshShape {
    vertices: Vec<[f32; 3]>,
    indices: Vec<[u32; 3]>,
}

impl TriMeshShape {
    pub(crate) fn vertices(&self) -> &[[f32; 3]] {
        self.vertices.as_slice()
    }

    pub(crate) fn indices(&self) -> &[[u32; 3]] {
        self.indices.as_slice()
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct LaserCannonInfo {
    muzzle: [f32; 3],
    range: f32,
    damage: f32,
    charge_time_sec: f32,
    discharge_time_sec: f32,
}

impl LaserCannonInfo {
    pub(crate) fn muzzle(&self) -> &[f32; 3] {
        &self.muzzle
    }

    pub(crate) fn range(&self) -> f32 {
        self.range
    }

    pub(crate) fn damage(&self) -> f32 {
        self.damage
    }

    /// A time duration in seconds. The cannon takes this long to charge before
    /// firing.
    pub(crate) fn charge_time_sec(&self) -> f32 {
        self.charge_time_sec
    }

    /// A time duration in seconds. The cannon takes this long to fully
    /// discharge if not actively charged.
    pub(crate) fn discharge_time_sec(&self) -> f32 {
        self.discharge_time_sec
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct FlightInfo {
    min_height: f32,
    max_height: f32,
}

impl FlightInfo {
    pub(crate) fn min_height(&self) -> f32 {
        self.min_height
    }

    pub(crate) fn max_height(&self) -> f32 {
        self.max_height
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct FactoryInfo {
    products: HashSet<UnitType>,
    position: [f32; 2],
    gate: [f32; 2],
}

impl FactoryInfo {
    pub(crate) fn products(&self) -> &HashSet<UnitType> {
        &self.products
    }

    pub(crate) fn position(&self) -> [f32; 2] {
        self.position
    }

    pub(crate) fn gate(&self) -> [f32; 2] {
        self.gate
    }
}

pub(crate) struct ObjectLoader;

impl AssetLoader for ObjectLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let object_info: ObjectInfo =
                serde_json::from_slice(bytes).context("Failed to parse object JSON")?;
            load_context.set_default_asset(LoadedAsset::new(object_info));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        OBJECT_EXTENSION.as_slice()
    }
}
