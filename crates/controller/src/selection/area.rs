use bevy::prelude::*;
use de_core::{
    frustum,
    gamestate::GameState,
    objects::{ObjectType, Playable},
    screengeom::ScreenRect,
    stages::GameStage,
};
use de_objects::{ColliderCache, ObjectCache};
use iyes_loopless::prelude::*;

use crate::{
    frustum::ScreenFrustum,
    selection::{SelectEvent, SelectionLabels, SelectionMode},
};

pub(super) struct AreaPlugin;

impl Plugin for AreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SelectInRectEvent>()
            .add_system_set_to_stage(
                GameStage::Input,
                SystemSet::new().with_system(
                    select_in_area
                        .run_in_state(GameState::Playing)
                        .label(AreaSelectLabels::SelectInArea)
                        .before(SelectionLabels::Update),
                ),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum AreaSelectLabels {
    SelectInArea,
}

pub(crate) struct SelectInRectEvent {
    rect: ScreenRect,
    mode: SelectionMode,
    filter_object_type: Option<ObjectType>,
}

impl SelectInRectEvent {
    pub(crate) fn new(
        rect: ScreenRect,
        mode: SelectionMode,
        filter_object_type: Option<ObjectType>,
    ) -> Self {
        Self {
            rect,
            mode,
            filter_object_type,
        }
    }

    fn rect(&self) -> ScreenRect {
        self.rect
    }

    fn mode(&self) -> SelectionMode {
        self.mode
    }

    fn filter_object_type(&self) -> Option<ObjectType> {
        self.filter_object_type
    }
}

fn select_in_area(
    screen_frustum: ScreenFrustum,
    cache: Res<ObjectCache>,
    candidates: Query<(Entity, &ObjectType, &Transform), With<Playable>>,
    mut in_events: EventReader<SelectInRectEvent>,
    mut out_events: EventWriter<SelectEvent>,
) {
    for in_event in in_events.iter() {
        let event_frustum = screen_frustum.rect(in_event.rect());
        let entities: Vec<Entity> = candidates
            .iter()
            .filter(|(_, &object_type, _)| {
                in_event
                    .filter_object_type()
                    .map_or(true, |filter| filter == object_type)
            })
            .filter_map(|(entity, &object_type, &transform)| {
                let aabb = cache.get_collider(object_type).aabb();
                if frustum::intersects_parry(&event_frustum, transform, &aabb) {
                    Some(entity)
                } else {
                    None
                }
            })
            .collect();
        out_events.send(SelectEvent::many(entities, in_event.mode()));
    }
}
