use bevy::{asset::LoadState, prelude::*};
use bevy_kira_audio::AudioSource as KAudioSource;
use de_audio::spatial::SpatialSoundBundle;
use de_core::{
    baseset::GameSet,
    cleanup::DespawnOnGameExit,
    gamestate::GameState,
    gconfig::GameConfig,
    objects::{BuildingType, ObjectType},
    state::AppState,
};
use de_spawner::{DraftAllowed, DraftBundle, SpawnBundle};
use iyes_progress::{Progress, ProgressSystem};

use crate::mouse::{Pointer, PointerSet};

pub(crate) struct DraftPlugin;

impl Plugin for DraftPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnDraftsEvent>()
            .add_event::<NewDraftEvent>()
            .add_event::<DiscardDraftsEvent>()
            .add_system(setup.in_schedule(OnEnter(AppState::AppLoading)))
            .add_system(load.track_progress().run_if(in_state(AppState::AppLoading)))
            .add_system(
                spawn
                    .in_base_set(GameSet::Input)
                    .run_if(in_state(AppState::InGame))
                    .run_if(on_event::<SpawnDraftsEvent>())
                    .in_set(DraftSet::Spawn),
            )
            .add_system(
                new_drafts
                    .in_base_set(GameSet::Input)
                    .run_if(in_state(AppState::InGame))
                    .in_set(DraftSet::New),
            )
            .add_system(
                discard_drafts
                    .in_base_set(GameSet::Input)
                    .run_if(in_state(AppState::InGame))
                    .run_if(on_event::<DiscardDraftsEvent>())
                    .in_set(DraftSet::Discard),
            )
            .add_system(
                move_drafts
                    .in_base_set(GameSet::Input)
                    .run_if(in_state(GameState::Playing))
                    .after(PointerSet::Update),
            );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub(crate) enum DraftSet {
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

#[derive(Resource)]
struct ConstructionSound(Handle<KAudioSource>);

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(ConstructionSound(server.load("audio/sounds/construct.ogg")));
}

fn load(server: Res<AssetServer>, sound: Res<ConstructionSound>) -> Progress {
    match server.get_load_state(&sound.0) {
        LoadState::Loaded => true.into(),
        LoadState::NotLoaded | LoadState::Loading => false.into(),
        _ => panic!("Unexpected loading state."),
    }
}

fn spawn(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    drafts: Query<(Entity, &Transform, &ObjectType, &DraftAllowed)>,
    sound: Res<ConstructionSound>,
) {
    for (entity, &transform, &object_type, draft) in drafts.iter() {
        if draft.allowed() {
            commands.entity(entity).despawn_recursive();
            commands.spawn((
                SpawnBundle::new(object_type, transform),
                game_config.locals().playable(),
                DespawnOnGameExit,
            ));

            commands.spawn((
                TransformBundle::from_transform(transform),
                SpatialSoundBundle {
                    sound: sound.0.clone(),
                    ..Default::default()
                },
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
