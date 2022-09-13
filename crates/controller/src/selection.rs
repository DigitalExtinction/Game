use std::collections::HashSet;

use bevy::{
    ecs::system::SystemParam,
    prelude::{App, Commands, Component, Entity, EventReader, Plugin, Query, With},
};
use de_core::{stages::GameStage, state::GameState};
use iyes_loopless::prelude::*;

use crate::Labels;

pub(crate) struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SelectEvent>().add_system_to_stage(
            GameStage::Input,
            update_selection
                .run_in_state(GameState::Playing)
                .after(Labels::InputUpdate),
        );
    }
}

pub(crate) struct SelectEvent {
    entities: Vec<Entity>,
    mode: SelectionMode,
}

impl SelectEvent {
    pub(crate) fn none(mode: SelectionMode) -> Self {
        Self {
            entities: Vec::new(),
            mode,
        }
    }

    pub(crate) fn single(entity: Entity, mode: SelectionMode) -> Self {
        Self {
            entities: vec![entity],
            mode,
        }
    }

    fn entities(&self) -> &[Entity] {
        self.entities.as_slice()
    }

    fn mode(&self) -> SelectionMode {
        self.mode
    }
}

#[derive(Component)]
pub(crate) struct Selected;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum SelectionMode {
    Replace,
    Add,
}

#[derive(SystemParam)]
struct Selector<'w, 's> {
    commands: Commands<'w, 's>,
    selected: Query<'w, 's, Entity, With<Selected>>,
}

impl<'w, 's> Selector<'w, 's> {
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

fn update_selection(mut events: EventReader<SelectEvent>, mut selector: Selector) {
    for event in events.iter() {
        selector.select(event.entities(), event.mode());
    }
}
