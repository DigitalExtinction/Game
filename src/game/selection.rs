use super::{pointer::Pointer, GameStates, Labels};
use bevy::{
    ecs::system::SystemParam,
    input::{mouse::MouseButtonInput, ElementState, Input},
    prelude::{
        App, Commands, Component, Entity, EventReader, KeyCode, MouseButton,
        ParallelSystemDescriptorCoercion, Plugin, Query, Res, SystemSet, With,
    },
};
use std::collections::HashSet;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameStates::Playing)
                .with_system(mouse_click_handler.after(Labels::PreInputUpdate)),
        );
    }
}

#[derive(Component)]
pub struct Selected;

#[derive(Clone, Copy, PartialEq)]
enum SelectionMode {
    Replace,
    Add,
}

#[derive(SystemParam)]
struct Selector<'w, 's> {
    commands: Commands<'w, 's>,
    selected: Query<'w, 's, Entity, With<Selected>>,
}

impl<'w, 's> Selector<'w, 's> {
    fn select_single(&mut self, entity: Option<Entity>, mode: SelectionMode) {
        let entities = match entity {
            Some(entity) => vec![entity],
            None => Vec::new(),
        };
        self.select(&entities, mode);
    }

    fn select(&mut self, entities: &[Entity], mode: SelectionMode) {
        let selected: HashSet<Entity> = self.selected.iter().collect();
        let desired: HashSet<Entity> = entities.iter().cloned().collect();

        if mode == SelectionMode::Replace {
            for deselect in &selected - &desired {
                self.commands.entity(deselect).remove::<Selected>();
            }
        }
        for select in &desired - &selected {
            self.commands.entity(select).insert(Selected);
        }
    }
}

fn mouse_click_handler(
    mut event: EventReader<MouseButtonInput>,
    keys: Res<Input<KeyCode>>,
    pointer: Res<Pointer>,
    mut selector: Selector,
) {
    if !event
        .iter()
        .any(|e| e.button == MouseButton::Left && e.state == ElementState::Pressed)
    {
        return;
    }

    let mode = if keys.pressed(KeyCode::LControl) {
        SelectionMode::Add
    } else {
        SelectionMode::Replace
    };

    selector.select_single(pointer.entity(), mode);
}
