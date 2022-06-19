use anyhow::{Context, Result};
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    reflect::TypeUuid,
};
use serde::{Deserialize, Serialize};

const OBJECT_EXTENSION: [&str; 1] = ["obj.json"];

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "5f938388-ebe1-4bb2-bb66-f3e182e4e0bb"]
pub(crate) struct Footprint {
    vertices: Vec<[f32; 2]>,
}

impl Footprint {
    pub(crate) fn vertices(&self) -> &[[f32; 2]] {
        self.vertices.as_slice()
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
            let footprint: Footprint =
                serde_json::from_slice(bytes).context("Failed to parse object JSON")?;
            load_context.set_default_asset(LoadedAsset::new(footprint));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        OBJECT_EXTENSION.as_slice()
    }
}
