use bevy::prelude::*;
use de_core::{
    frustum,
    objects::{ObjectType, Playable},
    screengeom::ScreenRect,
    stages::GameStage,
    state::GameState,
};
use de_objects::{ColliderCache, ObjectCache};
use iyes_loopless::prelude::*;

use crate::{
    frustum::ScreenFrustum,
    selection::{SelectEvent, SelectionLabels, SelectionMode},
};

pub(crate) struct AreaSelectPlugin;

impl Plugin for AreaSelectPlugin {
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
}

impl SelectInRectEvent {
    pub(crate) fn new(rect: ScreenRect, mode: SelectionMode) -> Self {
        Self { rect, mode }
    }

    fn rect(&self) -> ScreenRect {
        self.rect
    }

    fn mode(&self) -> SelectionMode {
        self.mode
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
