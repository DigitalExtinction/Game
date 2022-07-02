use bevy::prelude::*;
use de_core::{projection::ToFlat, state::GameState};
use de_objects::LaserCannon;
use de_pathing::PathTarget;
use de_pathing::{EntityPathSchedule, UpdateEntityPath};
use iyes_loopless::prelude::*;

pub(crate) struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::new().with_system(chase.run_in_state(GameState::Playing)),
        );
    }
}

/// Add this component to any attack-capable entity to chase an attack a target
/// entity.
#[derive(Component)]
pub struct AttackTarget(Entity);

impl AttackTarget {
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }

    pub fn entity(&self) -> Entity {
        self.0
    }
}

fn chase(
    mut commands: Commands,
    mut path_events: EventWriter<UpdateEntityPath>,
    pathing: EntityPathSchedule,
    attackers: Query<(Entity, &GlobalTransform, &LaserCannon, &AttackTarget)>,
    targets: Query<&GlobalTransform>,
) {
    for (attacker, attacker_transform, cannon, target) in attackers.iter() {
        let target_transform = match targets.get(target.entity()) {
            Ok(transform) => transform,
            Err(_) => {
                // TODO what happens if attacker gets despawned in the meantime?
                commands.entity(attacker).remove::<AttackTarget>();
                continue;
            }
        };

        let attacker_position = attacker_transform.translation.to_flat();
        let target_position = target_transform.translation.to_flat();

        let (path_target, distance) = pathing
            .scheduled_path_target(attacker)
            .or_else(|| pathing.current_path_target(attacker))
            .map(|path_target| (path_target.location(), path_target.distance()))
            .unwrap_or((attacker_position, 0.));

        // TODO put to a constant
        if (target_position - path_target).length() + distance <= 0.7 * cannon.range() {
            continue;
        }

        // TODO put to a constant
        path_events.send(UpdateEntityPath::new(
            attacker,
            PathTarget::new(target_position, 0.3 * cannon.range()),
        ));
    }
}
