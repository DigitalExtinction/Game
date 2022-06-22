use bevy::{input::mouse::MouseButtonInput, prelude::*};
use de_core::{objects::MovableSolid, projection::ToFlat};
use de_pathing::UpdateEntityPath;
use iyes_loopless::prelude::*;

use crate::{pointer::Pointer, selection::Selected, Labels};

pub(crate) struct CommandPlugin;

impl Plugin for CommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new().with_system(
                right_click_handler
                    .run_if(on_click(MouseButton::Right))
                    .label(Labels::InputUpdate)
                    .after(Labels::PreInputUpdate),
            ),
        );
    }
}

fn on_click(button: MouseButton) -> impl Fn(EventReader<MouseButtonInput>) -> bool {
    move |mut events: EventReader<MouseButtonInput>| {
        // It is desirable to exhaust the iterator, thus .filter().count() is
        // used instead of .any()
        events.iter().filter(|e| e.button == button).count() > 0
    }
}

fn right_click_handler(
    mut path_events: EventWriter<UpdateEntityPath>,
    selected: Query<Entity, (With<Selected>, With<MovableSolid>)>,
    pointer: Res<Pointer>,
) {
    let target = match pointer.terrain_point() {
        Some(point) => point.to_flat(),
        None => return,
    };

    for entity in selected.iter() {
        path_events.send(UpdateEntityPath::new(entity, target));
    }
}
