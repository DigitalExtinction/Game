use ahash::AHashSet;
use bevy::utils::HashSet;
use de_types::objects::UnitType;
use glam::Vec2;
use serde::{Deserialize, Serialize};

pub struct Factory {
    products: AHashSet<UnitType>,
    position: Vec2,
}

impl Factory {
    pub fn products(&self) -> &AHashSet<UnitType> {
        &self.products
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }
}

impl TryFrom<FactorySerde> for Factory {
    type Error = anyhow::Error;

    fn try_from(factory_serde: FactorySerde) -> Result<Self, Self::Error> {
        Ok(Self {
            products: AHashSet::from_iter(factory_serde.products),
            position: factory_serde.position.into(),
        })
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct FactorySerde {
    products: HashSet<UnitType>,
    position: [f32; 2],
}
