use bevy::prelude::*;

use de_core::objects::BuildingType;
use de_spawner::ObjectCounter;
use crate::{
    command::place_draft_system,
    draft::NewDraftEvent,
    pointer::Pointer,
};

#[derive(Component, Clone, Copy)]
pub(crate) enum HudButtonAction {
    Build(BuildingType),
}

pub(crate) fn hud_button_system(
    interactions: Query<(&Interaction, &HudButtonAction), Changed<Interaction>>,
    counter: Res<ObjectCounter>,
    pointer: Res<Pointer>,
    events: EventWriter<NewDraftEvent>,
) {
    for (&interaction, &action) in interactions.iter() {
        if let Interaction::Clicked = interaction {
            match action {
                HudButtonAction::Build(building) => {
                    place_draft_system(building, counter, pointer, events);
                    return;
                }
            };
        }
    }
}
