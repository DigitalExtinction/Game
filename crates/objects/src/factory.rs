use ahash::AHashSet;
use de_core::objects::UnitType;
use glam::Vec2;

use crate::loader::FactoryInfo;

pub struct Factory {
    products: AHashSet<UnitType>,
    position: Vec2,
    gate: Vec2,
}

impl Factory {
    pub fn products(&self) -> &AHashSet<UnitType> {
        &self.products
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn gate(&self) -> Vec2 {
        self.gate
    }
}

impl From<&FactoryInfo> for Factory {
    fn from(info: &FactoryInfo) -> Self {
        Self {
            products: AHashSet::from_iter(info.products().iter().cloned()),
            position: info.position().into(),
            gate: info.gate().into(),
        }
    }
}
