use bevy::prelude::*;
use de_audio::spatial::{PlaySpatialAudioEvent, Sound};
use de_core::{
    cleanup::DespawnOnGameExit, gamestate::GameState, gconfig::GameConfig,
    objects::ObjectTypeComponent, player::PlayerComponent, schedule::InputSchedule,
    state::AppState,
};
use de_spawner::{DraftAllowed, DraftBundle, SpawnBundle};
use de_types::objects::BuildingType;

use crate::mouse::{Pointer, PointerSet};

pub(crate) struct DraftPlugin;

impl Plugin for DraftPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnDraftsEvent>()
            .add_event::<NewDraftEvent>()
            .add_event::<DiscardDraftsEvent>()
            .add_systems(
                InputSchedule,
                (
                    (
                        spawn
                            .run_if(on_event::<SpawnDraftsEvent>())
                            .in_set(DraftSet::Spawn),
                        new_drafts.in_set(DraftSet::New),
                        discard_drafts
                            .run_if(on_event::<DiscardDraftsEvent>())
                            .in_set(DraftSet::Discard),
                    )
                        .run_if(in_state(AppState::InGame)),
                    move_drafts
                        .run_if(in_state(GameState::Playing))
                        .after(PointerSet::Update),
                ),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub(crate) enum DraftSet {
    Spawn,
    New,
    Discard,
}

#[derive(Event)]
pub(crate) struct SpawnDraftsEvent;

#[derive(Event)]
pub(crate) struct NewDraftEvent {
    point: Vec3,
    building_type: BuildingType,
}

#[derive(Event)]
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
    drafts: Query<(Entity, &Transform, &ObjectTypeComponent, &DraftAllowed)>,
    mut play_audio: EventWriter<PlaySpatialAudioEvent>,
) {
    for (entity, &transform, &object_type, draft) in drafts.iter() {
        if draft.allowed() {
            commands.entity(entity).despawn_recursive();
            commands.spawn((
                SpawnBundle::new(*object_type, transform),
                PlayerComponent::from(game_config.locals().playable()),
                DespawnOnGameExit,
            ));

            play_audio.send(PlaySpatialAudioEvent::new(
                Sound::Construct,
                transform.translation,
            ));
        }
    }
}

fn new_drafts(
    mut commands: Commands,
    mut events: EventReader<NewDraftEvent>,
    drafts: Query<Entity, With<DraftAllowed>>,
) {
    let event = match events.iter().last() {
        Some(event) => event,
        None => return,
    };

    for entity in drafts.iter() {
        commands.entity(entity).despawn_recursive();
    }

    commands.spawn((
        DraftBundle::new(
            event.building_type(),
            Transform {
                translation: event.point(),
                ..Default::default()
            },
        ),
        DespawnOnGameExit,
    ));
}

fn discard_drafts(mut commands: Commands, drafts: Query<Entity, With<DraftAllowed>>) {
    for entity in drafts.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn move_drafts(pointer: Res<Pointer>, mut drafts: Query<&mut Transform, With<DraftAllowed>>) {
    let pointer_changed = pointer.is_changed();

    let point = match pointer.terrain_point() {
        Some(point) => point,
        None => return,
    };

    for mut transform in drafts.iter_mut() {
        if transform.is_added() || pointer_changed {
            transform.translation = point;
        }
    }
}
