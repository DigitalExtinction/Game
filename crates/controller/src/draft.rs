use bevy::prelude::*;
use de_core::{
    gconfig::GameConfig,
    objects::{ActiveObjectType, ObjectType},
};
use de_spawner::{Draft, DraftBundle, SpawnBundle};

use super::Labels;
use crate::pointer::Pointer;

pub(crate) struct DraftPlugin;

impl Plugin for DraftPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnDraftsEvent>()
            .add_event::<NewDraftEvent>()
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_system(spawn.after(Labels::InputUpdate))
                    .with_system(new_drafts.after(Labels::InputUpdate))
                    .with_system(
                        move_drafts
                            .label(Labels::InputUpdate)
                            .after(Labels::PreInputUpdate),
                    ),
            );
    }
}

pub(crate) struct SpawnDraftsEvent;

pub(crate) struct NewDraftEvent {
    point: Vec3,
    object_type: ActiveObjectType,
}

impl NewDraftEvent {
    pub(crate) fn new(point: Vec3, object_type: ActiveObjectType) -> Self {
        Self { point, object_type }
    }

    fn point(&self) -> Vec3 {
        self.point
    }

    fn object_type(&self) -> ActiveObjectType {
        self.object_type
    }
}

fn spawn(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    mut events: EventReader<SpawnDraftsEvent>,
    // Use global transform, since that is the one rendered in the last frame
    // (and seen by the user) and checked for collisions.
    drafts: Query<(Entity, &GlobalTransform, &ObjectType, &Draft)>,
) {
    if events.iter().count() == 0 {
        return;
    }

    for (entity, &transform, &object_type, draft) in drafts.iter() {
        if draft.allowed() {
            commands.entity(entity).despawn_recursive();
            commands
                .spawn_bundle(SpawnBundle::new(object_type, transform.into()))
                .insert(game_config.player());
        }
    }
}

fn new_drafts(
    mut commands: Commands,
    mut events: EventReader<NewDraftEvent>,
    drafts: Query<Entity, With<Draft>>,
) {
    let event = match events.iter().last() {
        Some(event) => event,
        None => return,
    };

    for entity in drafts.iter() {
        // TODO: this sometimes leads to an error:
        // Entity 1625v23 does not exist
        commands.entity(entity).despawn_recursive();
    }

    commands.spawn_bundle(DraftBundle::new(
        event.object_type(),
        Transform {
            translation: event.point(),
            ..Default::default()
        },
    ));
}

fn move_drafts(pointer: Res<Pointer>, mut drafts: Query<&mut Transform, With<Draft>>) {
    // TODO move by some delta, rather than to a point
    // TODO: make this a no-op if the mouse was not moved
    let point = match pointer.terrain_point() {
        Some(point) => point,
        None => return,
    };
    for mut transform in drafts.iter_mut() {
        transform.translation = point;
    }
}
