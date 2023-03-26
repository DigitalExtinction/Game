use de_core::objects::{ActiveObjectType, BuildingType, InactiveObjectType, ObjectType, UnitType};

pub(crate) trait FileStem: Copy {
    fn stem(self) -> &'static str;
}

impl FileStem for ObjectType {
    fn stem(self) -> &'static str {
        match self {
            Self::Active(ActiveObjectType::Building(BuildingType::Base)) => "base",
            Self::Active(ActiveObjectType::Building(BuildingType::PowerHub)) => "powerhub",
            Self::Active(ActiveObjectType::Unit(UnitType::Attacker)) => "attacker",
            Self::Inactive(InactiveObjectType::Tree) => "tree",
        }
    }
}
