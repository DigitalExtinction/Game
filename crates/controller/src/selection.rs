use ahash::AHashSet;
use bevy::{ecs::system::SystemParam, prelude::*};
use de_core::{stages::GameStage, state::GameState};
use de_signs::UpdateBarVisibilityEvent;
use de_terrain::CircleMarker;
use iyes_loopless::prelude::*;

use crate::SELECTION_BAR_ID;

pub(crate) struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SelectEvent>().add_system_to_stage(
            GameStage::Input,
            update_selection
                .run_in_state(GameState::Playing)
                .label(SelectionLabels::Update),
        );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum SelectionLabels {
    Update,
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

    pub(crate) fn many(entities: Vec<Entity>, mode: SelectionMode) -> Self {
        Self { entities, mode }
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
    /// Selected entities are union of currently selected and to be selected
    /// entities.
    Add,
    /// Toggle selection for all updated entities, and keep other entities
    /// untouched.
    AddToggle,
}

#[derive(SystemParam)]
struct Selector<'w, 's> {
    commands: Commands<'w, 's>,
    selected: Query<'w, 's, Entity, With<Selected>>,
    markers: Query<'w, 's, &'static mut CircleMarker>,
    bars: EventWriter<'w, 's, UpdateBarVisibilityEvent>,
}

impl<'w, 's> Selector<'w, 's> {
    fn select(&mut self, entities: &[Entity], mode: SelectionMode) {
        let selected: AHashSet<Entity> = self.selected.iter().collect();
        let updated: AHashSet<Entity> = entities.iter().cloned().collect();

        let (select, deselect): (AHashSet<Entity>, AHashSet<Entity>) = match mode {
            SelectionMode::Replace => (&updated - &selected, &selected - &updated),
            SelectionMode::AddToggle => (&updated - &selected, &updated & &selected),
            SelectionMode::Add => (&updated - &selected, AHashSet::new()),
        };

        for entity in deselect {
            let mut entity_commands = self.commands.entity(entity);
            entity_commands.remove::<Selected>();

            if let Ok(mut marker) = self.markers.get_mut(entity) {
                marker
                    .visibility_mut()
                    .update_visible(SELECTION_BAR_ID, false);
            }

            self.bars.send(UpdateBarVisibilityEvent::new(
                entity,
                SELECTION_BAR_ID,
                false,
            ));
        }

        for entity in select {
            let mut entity_commands = self.commands.entity(entity);
            entity_commands.insert(Selected);

            if let Ok(mut marker) = self.markers.get_mut(entity) {
                marker
                    .visibility_mut()
                    .update_visible(SELECTION_BAR_ID, true);
            }

            self.bars.send(UpdateBarVisibilityEvent::new(
                entity,
                SELECTION_BAR_ID,
                true,
            ));
        }
    }
}

fn update_selection(mut events: EventReader<SelectEvent>, mut selector: Selector) {
    for event in events.iter() {
        selector.select(event.entities(), event.mode());
    }
}
