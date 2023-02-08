use bevy::prelude::*;
use de_behaviour::ChaseTargetEvent;
use de_combat::AttackEvent;
use de_core::{gamestate::GameState, objects::MovableSolid, stages::GameStage};
use de_pathing::{PathQueryProps, PathTarget, UpdateEntityPath};
use glam::Vec2;
use iyes_loopless::prelude::*;

use crate::selection::Selected;

pub(super) struct ExecutorPlugin;

impl Plugin for ExecutorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SendSelectedEvent>()
            .add_event::<GroupAttackEvent>()
            .add_system_set_to_stage(
                GameStage::Input,
                SystemSet::new()
                    .with_system(
                        send_selected_system
                            .run_in_state(GameState::Playing)
                            .label(CommandsLabel::SendSelected),
                    )
                    .with_system(
                        attack_system
                            .run_in_state(GameState::Playing)
                            .label(CommandsLabel::Attack),
                    ),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum CommandsLabel {
    SendSelected,
    Attack,
}

/// Send this event to send all selected movable units to a point on the map.
pub(crate) struct SendSelectedEvent(Vec2);

impl SendSelectedEvent {
    pub(crate) fn new(target: Vec2) -> Self {
        Self(target)
    }

    fn target(&self) -> Vec2 {
        self.0
    }
}

/// Send this event to attack an enemy with all selected movable units. The
/// target must be an enemy entity.
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
    mut path_events: EventWriter<UpdateEntityPath>,
    mut chase_events: EventWriter<ChaseTargetEvent>,
) {
    if let Some(send) = send_events.iter().last() {
        for entity in selected.iter() {
            chase_events.send(ChaseTargetEvent::new(entity, None));
            path_events.send(UpdateEntityPath::new(
                entity,
                PathTarget::new(send.target(), PathQueryProps::exact(), false),
            ));
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
