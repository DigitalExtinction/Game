use ahash::AHashSet;
use bevy::{ecs::system::SystemParam, prelude::*};
use de_core::{baseset::GameSet, gamestate::GameState};
use de_signs::{UpdateBarVisibilityEvent, UpdatePoleVisibilityEvent};
use de_terrain::MarkerVisibility;

use crate::SELECTION_BAR_ID;

pub(super) struct BookkeepingPlugin;

impl Plugin for BookkeepingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SelectEvent>()
            .add_event::<SelectedEvent>()
            .add_event::<DeselectedEvent>()
            .add_system(
                update_selection
                    .in_base_set(GameSet::Input)
                    .run_if(in_state(GameState::Playing))
                    .in_set(SelectionSet::Update),
            )
            .add_system(
                selected_system
                    .in_base_set(GameSet::Input)
                    .run_if(in_state(GameState::Playing))
                    .after(SelectionSet::Update),
            )
            .add_system(
                deselected_system
                    .in_base_set(GameSet::Input)
                    .run_if(in_state(GameState::Playing))
                    .after(SelectionSet::Update),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub(crate) enum SelectionSet {
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

struct SelectedEvent(Entity);

struct DeselectedEvent(Entity);

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
struct SelectorBuilder<'w, 's> {
    commands: Commands<'w, 's>,
    selected: Query<'w, 's, Entity, With<Selected>>,
    selected_events: EventWriter<'w, SelectedEvent>,
    deselected_events: EventWriter<'w, DeselectedEvent>,
}

impl<'w, 's> SelectorBuilder<'w, 's> {
    fn build(self) -> Selector<'w, 's> {
        let selected: AHashSet<Entity> = self.selected.iter().collect();
        Selector {
            commands: self.commands,
            selected,
            to_select: AHashSet::new(),
            to_deselect: AHashSet::new(),
            selected_events: self.selected_events,
            deselected_events: self.deselected_events,
        }
    }
}

struct Selector<'w, 's> {
    commands: Commands<'w, 's>,
    selected: AHashSet<Entity>,
    to_select: AHashSet<Entity>,
    to_deselect: AHashSet<Entity>,
    selected_events: EventWriter<'w, SelectedEvent>,
    deselected_events: EventWriter<'w, DeselectedEvent>,
}

impl<'w, 's> Selector<'w, 's> {
    fn update(&mut self, entities: &[Entity], mode: SelectionMode) {
        let updated: AHashSet<Entity> = entities.iter().cloned().collect();

        match mode {
            SelectionMode::Replace => {
                self.to_select = &updated - &self.selected;
                self.to_deselect = &self.selected - &updated;
            }
            SelectionMode::AddToggle => {
                self.to_select = &(&updated - &self.to_select) - &self.selected;
                self.to_deselect = &updated & &self.selected;
            }
            SelectionMode::Add => {
                self.to_select.extend(&updated - &self.selected);
                self.to_deselect = &self.to_deselect - &updated;
            }
        }
    }

    fn execute(mut self) {
        for entity in self.to_deselect {
            self.commands.entity(entity).remove::<Selected>();
            self.deselected_events.send(DeselectedEvent(entity));
        }

        for entity in self.to_select {
            self.commands.entity(entity).insert(Selected);
            self.selected_events.send(SelectedEvent(entity));
        }
    }
}

fn update_selection(mut events: EventReader<SelectEvent>, selector_builder: SelectorBuilder) {
    let mut selector = selector_builder.build();
    for event in events.iter() {
        selector.update(event.entities(), event.mode());
    }
    selector.execute();
}

fn selected_system(
    mut events: EventReader<SelectedEvent>,
    mut markers: Query<&mut MarkerVisibility>,
    mut bars: EventWriter<UpdateBarVisibilityEvent>,
    mut poles: EventWriter<UpdatePoleVisibilityEvent>,
) {
    for event in events.iter() {
        if let Ok(mut visibility) = markers.get_mut(event.0) {
            visibility.0.update_visible(SELECTION_BAR_ID, true);
        }

        bars.send(UpdateBarVisibilityEvent::new(
            event.0,
            SELECTION_BAR_ID,
            true,
        ));

        poles.send(UpdatePoleVisibilityEvent::new(event.0, true));
    }
}

fn deselected_system(
    mut events: EventReader<DeselectedEvent>,
    mut markers: Query<&mut MarkerVisibility>,
    mut bars: EventWriter<UpdateBarVisibilityEvent>,
    mut poles: EventWriter<UpdatePoleVisibilityEvent>,
) {
    for event in events.iter() {
        if let Ok(mut visibility) = markers.get_mut(event.0) {
            visibility.0.update_visible(SELECTION_BAR_ID, false);
        }

        bars.send(UpdateBarVisibilityEvent::new(
            event.0,
            SELECTION_BAR_ID,
            false,
        ));

        poles.send(UpdatePoleVisibilityEvent::new(event.0, false));
    }
}
