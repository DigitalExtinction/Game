use bevy::prelude::*;
use de_core::{
    gconfig::GameConfig,
    objects::{BuildingType, ObjectType},
    stages::GameStage,
};
use de_spawner::{Draft, DraftBundle, SpawnBundle};

use crate::pointer::{Pointer, PointerLabels};

pub(crate) struct DraftPlugin;

impl Plugin for DraftPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnDraftsEvent>()
            .add_event::<NewDraftEvent>()
            .add_event::<DiscardDraftsEvent>()
            .add_system_set_to_stage(
                GameStage::Input,
                SystemSet::new()
                    .with_system(spawn.label(DraftLabels::Spawn))
                    .with_system(new_drafts.label(DraftLabels::New))
                    .with_system(discard_drafts.label(DraftLabels::Discard))
                    .with_system(move_drafts.after(PointerLabels::Update)),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum DraftLabels {
    Spawn,
    New,
    Discard,
}

pub(crate) struct SpawnDraftsEvent;

pub(crate) struct NewDraftEvent {
    point: Vec3,
    building_type: BuildingType,
}

pub(crate) struct DiscardDraftsEvent;

impl NewDraftEvent {
    pub(crate) fn new(point: Vec3, building_type: BuildingType) -> Self {
        Self {
            point,
            building_type,
        }
    }

    fn point(&self) -> Vec3 {
        self.point
    }

    fn building_type(&self) -> BuildingType {
        self.building_type
    }
}

fn spawn(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    mut events: EventReader<SpawnDraftsEvent>,
    drafts: Query<(Entity, &Transform, &ObjectType, &Draft)>,
) {
    if events.iter().count() == 0 {
        return;
    }

    for (entity, &transform, &object_type, draft) in drafts.iter() {
        if draft.allowed() {
            commands.entity(entity).despawn_recursive();
            commands
                .spawn_bundle(SpawnBundle::new(object_type, transform))
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
        commands.entity(entity).despawn_recursive();
    }

    commands.spawn_bundle(DraftBundle::new(
        event.building_type(),
        Transform {
            translation: event.point(),
            ..Default::default()
        },
    ));
}

fn discard_drafts(
    mut commands: Commands,
    mut events: EventReader<DiscardDraftsEvent>,
    drafts: Query<Entity, With<Draft>>,
) {
    if events.iter().count() == 0 {
        return;
    }
    for entity in drafts.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn move_drafts(pointer: Res<Pointer>, mut drafts: Query<&mut Transform, With<Draft>>) {
    let point = match pointer.terrain_point() {
        Some(point) => point,
        None => return,
    };
    for mut transform in drafts.iter_mut() {
        transform.translation = point;
    }
}
