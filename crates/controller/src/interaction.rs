use bevy::prelude::{Component, Input, Interaction, MouseButton, Query, ResMut, SystemLabel};

use de_core::objects::BuildingType;
use crate::{
    command::place_draft,
    interaction::BuildBuildingButton::BuildBuilding
};

#[derive(Component)]
pub(crate) enum BuildBuildingButton {
    BuildBuilding(BuildingType)
}

pub(crate) fn handle_item_click(
    query: Query<(&Interaction, &BuildBuildingButton)>,
    mut mouse: ResMut<Input<MouseButton>>,
) {
    query.for_each(|(interaction, button)| match interaction {
        Interaction::Clicked => match button {
            BuildBuilding(building_type) => {
                mouse.clear();
                place_draft(building_type.clone());
            }
        },
        _ => {}
    });
}


#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum HudInteraction {
    Click
}
