use bevy::prelude::*;
use de_behaviour::ChaseTargetEvent;
use de_combat::AttackEvent;
use de_construction::{AssemblyLine, ChangeDeliveryLocationEvent};
use de_core::{gamestate::GameState, objects::MovableSolid, schedule::InputSchedule};
use de_pathing::{PathQueryProps, PathTarget, UpdateEntityPathEvent};
use glam::Vec2;

use crate::selection::Selected;

pub(super) struct ExecutorPlugin;

impl Plugin for ExecutorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SendSelectedEvent>()
            .add_event::<DeliveryLocationSelectedEvent>()
            .add_event::<GroupAttackEvent>()
            .add_systems(
                InputSchedule,
                (
                    send_selected_system
                        .run_if(in_state(GameState::Playing))
                        .in_set(CommandsSet::SendSelected),
                    delivery_location_system
                        .run_if(in_state(GameState::Playing))
                        .in_set(CommandsSet::DeliveryLocation),
                    attack_system
                        .run_if(in_state(GameState::Playing))
                        .in_set(CommandsSet::Attack),
                ),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub(crate) enum CommandsSet {
    SendSelected,
    DeliveryLocation,
    Attack,
}

/// Send this event to send all selected movable units to a point on the map.
#[derive(Event)]
pub(crate) struct SendSelectedEvent(Vec2);

impl SendSelectedEvent {
    pub(crate) fn new(target: Vec2) -> Self {
        Self(target)
    }

    fn target(&self) -> Vec2 {
        self.0
    }
}

/// Send this event to set manufacturing delivery location for all selected
/// building with a factory.
#[derive(Event)]
pub(crate) struct DeliveryLocationSelectedEvent(Vec2);

impl DeliveryLocationSelectedEvent {
    pub(crate) fn new(target: Vec2) -> Self {
        Self(target)
    }

    fn target(&self) -> Vec2 {
        self.0
    }
}

/// Send this event to attack an enemy with all selected movable units. The
/// target must be an enemy entity.
#[derive(Event)]
pub(crate) struct GroupAttackEvent(Entity);

impl GroupAttackEvent {
    pub(crate) fn new(target: Entity) -> Self {
        Self(target)
    }

    fn target(&self) -> Entity {
        self.0
    }
}

type SelectedMovable = (With<Selected>, With<MovableSolid>);

fn send_selected_system(
    mut send_events: EventReader<SendSelectedEvent>,
    selected: Query<Entity, SelectedMovable>,
    mut path_events: EventWriter<UpdateEntityPathEvent>,
    mut chase_events: EventWriter<ChaseTargetEvent>,
) {
    if let Some(send) = send_events.iter().last() {
        for entity in selected.iter() {
            chase_events.send(ChaseTargetEvent::new(entity, None));
            path_events.send(UpdateEntityPathEvent::new(
                entity,
                PathTarget::new(send.target(), PathQueryProps::exact(), false),
            ));
        }
    }
}

type SelectedFactory = (With<Selected>, With<AssemblyLine>);

fn delivery_location_system(
    mut in_events: EventReader<DeliveryLocationSelectedEvent>,
    selected: Query<Entity, SelectedFactory>,
    mut out_events: EventWriter<ChangeDeliveryLocationEvent>,
) {
    if let Some(event) = in_events.iter().last() {
        for entity in selected.iter() {
            out_events.send(ChangeDeliveryLocationEvent::new(entity, event.target()));
        }
    }
}

fn attack_system(
    mut group_events: EventReader<GroupAttackEvent>,
    selected: Query<Entity, SelectedMovable>,
    mut individual_events: EventWriter<AttackEvent>,
) {
    if let Some(group_event) = group_events.iter().last() {
        for attacker in selected.iter() {
            individual_events.send(AttackEvent::new(attacker, group_event.target()));
        }
    }
}
