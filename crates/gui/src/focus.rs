//! This module implement entity selection / focusing based on user
//! interactions and events.

use bevy::{
    ecs::{
        query::{ReadOnlyWorldQuery, WorldQuery},
        system::SystemParam,
    },
    prelude::*,
    ui::UiSystem,
};

pub(crate) struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiFocus>()
            .add_event::<SetFocusEvent>()
            .add_systems(PreUpdate, focus_system.after(UiSystem::Focus));
    }
}

/// Send this event to (de)select an entity.
#[derive(Event)]
pub struct SetFocusEvent(Option<Entity>);

impl SetFocusEvent {
    pub fn some(entity: Entity) -> Self {
        Self(Some(entity))
    }
}

/// This system parameter implements the query of selected / focused UI
/// entities.
///
/// An entity can be (de)selected / (de)focused by multiple means:
///
/// * Selected via [`Interaction::Pressed`].
/// * (De)selected via [`SetFocusEvent`].
/// * Deselected by despawning.
/// * Deselected by clicking outside of it.
#[derive(SystemParam)]
pub(super) struct FocusedQuery<'w, 's, Q, F = ()>
where
    Q: WorldQuery + Sync + Send + 'static,
    F: ReadOnlyWorldQuery + Sync + Send + 'static,
{
    focus: Res<'w, UiFocus>,
    query: Query<'w, 's, Q, F>,
}

impl<'w, 's, Q, F> FocusedQuery<'w, 's, Q, F>
where
    Q: WorldQuery + Sync + Send + 'static,
    F: ReadOnlyWorldQuery + Sync + Send + 'static,
{
    pub(super) fn is_changed(&self) -> bool {
        self.focus.is_changed()
    }

    /// Returns the query item for previously selected entity, id est the
    /// entity selected before the current one.
    pub(super) fn get_previous_mut(&mut self) -> Option<<Q as WorldQuery>::Item<'_>> {
        self.get_mut(self.focus.previous)
    }

    /// Returns the query item for currently selected entity.
    pub(super) fn get_current_mut(&mut self) -> Option<<Q as WorldQuery>::Item<'_>> {
        self.get_mut(self.focus.current)
    }

    fn get_mut(&mut self, entity: Option<Entity>) -> Option<<Q as WorldQuery>::Item<'_>> {
        match entity {
            Some(entity) => match self.query.get_mut(entity) {
                Ok(item) => Some(item),
                Err(_) => None,
            },
            None => None,
        }
    }
}

#[derive(Resource, Default)]
pub(super) struct UiFocus {
    previous: Option<Entity>,
    current: Option<Entity>,
}

fn focus_system(
    mut focus: ResMut<UiFocus>,
    mut removals: RemovedComponents<Interaction>,
    mouse: Res<Input<MouseButton>>,
    touch: Res<Touches>,
    interactions: Query<(Entity, &Interaction), Changed<Interaction>>,
    mut events: EventReader<SetFocusEvent>,
) {
    let mut current = focus.current;

    if let Some(current_entity) = current {
        if removals.iter().any(|e| e == current_entity) {
            current = None;
        }
    }

    if mouse.just_pressed(MouseButton::Left) || touch.any_just_pressed() {
        current = None;
    }

    for (entity, &interaction) in interactions.iter() {
        if matches!(interaction, Interaction::Pressed) {
            current = Some(entity);
        }
    }

    if let Some(event) = events.iter().last() {
        current = event.0;
    }

    // do not when unnecessarily trigger change detection
    if focus.current != current {
        focus.previous = focus.current;
        focus.current = current;
    }

    if let Some(previous_entity) = focus.previous {
        if removals.iter().any(|e| e == previous_entity) {
            focus.previous = None;
        }
    }
}
