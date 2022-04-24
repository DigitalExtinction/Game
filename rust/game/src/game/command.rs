use super::{movement::SendEntityEvent, pointer::Pointer, selection::Selected, Labels};
use bevy::{
    input::mouse::MouseButtonInput,
    prelude::{
        App, Entity, EventReader, EventWriter, MouseButton, ParallelSystemDescriptorCoercion,
        Plugin, Query, Res, SystemSet, With,
    },
};
use de_objects::Movable;
use glam::Vec2;

pub struct CommandPlugin;

impl Plugin for CommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new().with_system(
                mouse_click_handler
                    .label(Labels::InputUpdate)
                    .after(Labels::PreInputUpdate),
            ),
        );
    }
}

fn mouse_click_handler(
    mut click_events: EventReader<MouseButtonInput>,
    mut send_entity_events: EventWriter<SendEntityEvent>,
    selected: Query<Entity, (With<Selected>, With<Movable>)>,
    pointer: Res<Pointer>,
) {
    if !click_events.iter().any(|e| e.button == MouseButton::Right) {
        return;
    }

    let target = match pointer.terrain_point() {
        Some(point) => Vec2::new(point.x, point.z),
        None => return,
    };

    for entity in selected.iter() {
        send_entity_events.send(SendEntityEvent::new(entity, target));
    }
}
