use std::path::{Path, PathBuf};

use async_std::{fs, io, stream::StreamExt};
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use de_core::{assets::asset_path, log_full_error, state::AppState};
use de_gui::{ButtonCommands, GuiCommands, OuterStyle};
use de_map::{
    io::{load_metadata, MapLoadingError, MAP_FILE_SUFFIX},
    meta::MapMetadata,
};
use futures_lite::future;
use thiserror::Error;

pub(crate) struct MapSelectionPlugin;

impl Plugin for MapSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<MapState>()
            .add_event::<SelectMapEvent>()
            .add_event::<MapSelectedEvent>()
            .add_system(setup.in_schedule(OnEnter(MapState::On)))
            .add_system(cleanup.in_schedule(OnExit(MapState::On)))
            .add_system(init_buttons.run_if(in_state(MapState::On)))
            .add_system(button_system.run_if(in_state(MapState::On)))
            .add_system(select_map_system.run_if(in_state(AppState::InMenu)));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
enum MapState {
    On,
    #[default]
    Off,
}

/// Send this event to display map selection on top of current UI.
pub(crate) struct SelectMapEvent;

/// This event is sent after a map is selected, just before menu state is
/// switched to a next state.
pub(crate) struct MapSelectedEvent {
    path: PathBuf,
    metadata: MapMetadata,
}

impl MapSelectedEvent {
    fn new(path: PathBuf, metadata: MapMetadata) -> Self {
        Self { path, metadata }
    }

    /// Path to the map on the local file system.
    pub(crate) fn path(&self) -> &Path {
        self.path.as_path()
    }

    /// Selected map metadata.
    pub(crate) fn metadata(&self) -> &MapMetadata {
        &self.metadata
    }
}

#[derive(Resource)]
struct PopUpNode(Entity);

#[derive(Resource)]
struct LoadingTask(Task<Result<Vec<MapEntry>, LoadingError>>);

#[derive(Component)]
struct MapEntry(PathBuf, MapMetadata);

impl MapEntry {
    fn new(path: PathBuf, meta: MapMetadata) -> Self {
        Self(path, meta)
    }

    fn path(&self) -> &Path {
        self.0.as_path()
    }

    fn metadata(&self) -> &MapMetadata {
        &self.1
    }
}

#[derive(Error, Debug)]
pub enum LoadingError {
    #[error(transparent)]
    Io { source: io::Error },
    #[error(transparent)]
    Map { source: MapLoadingError },
}

fn setup(mut commands: Commands) {
    let task = IoTaskPool::get().spawn(load_available_maps());
    commands.insert_resource(LoadingTask(task));

    let node_id = commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect::all(Val::Percent(0.)),
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                ..default()
            },
            background_color: Color::GRAY.into(),
            z_index: ZIndex::Local(10),
            ..default()
        })
        .id();
    commands.insert_resource(PopUpNode(node_id));
}

fn init_buttons(
    mut commands: GuiCommands,
    node: Res<PopUpNode>,
    task: Option<ResMut<LoadingTask>>,
) {
    let Some(mut task) = task else { return };
    let Some(result) = future::block_on(future::poll_once(&mut task.0)) else {
        return
    };

    let map_entries = match result {
        Ok(entries) => entries,
        Err(err) => {
            log_full_error!(err);
            panic!("{}", err);
        }
    };

    commands.remove_resource::<LoadingTask>();

    let column_node = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                size: Size::new(Val::Percent(25.), Val::Percent(100.)),
                margin: UiRect::all(Val::Auto),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .id();

    commands.entity(node.0).add_child(column_node);

    for map in map_entries {
        let button = map_button(&mut commands, map);
        commands.entity(column_node).add_child(button);
    }
}

fn cleanup(mut commands: Commands, node: Res<PopUpNode>) {
    commands.remove_resource::<LoadingTask>();
    commands.entity(node.0).despawn_recursive();
}

fn button_system(
    mut next_state: ResMut<NextState<MapState>>,
    interactions: Query<(&Interaction, &MapEntry), Changed<Interaction>>,
    mut events: EventWriter<MapSelectedEvent>,
) {
    for (&interaction, map) in interactions.iter() {
        if let Interaction::Clicked = interaction {
            next_state.set(MapState::Off);
            events.send(MapSelectedEvent::new(
                map.path().into(),
                map.metadata().clone(),
            ));
        }
    }
}

async fn load_available_maps() -> Result<Vec<MapEntry>, LoadingError> {
    let maps_dir = asset_path("maps");

    let mut map_entries = Vec::new();
    let mut dir_entries = match fs::read_dir(maps_dir).await {
        Ok(entries) => entries,
        Err(err) => return Err(LoadingError::Io { source: err }),
    };

    while let Some(res) = dir_entries.next().await {
        let path = match res {
            Ok(entry) => entry.path(),
            Err(err) => return Err(LoadingError::Io { source: err }),
        };

        if !path.is_file().await {
            continue;
        }
        if !path
            .file_name()
            .and_then(|n| n.to_str())
            .map_or(false, |n| n.ends_with(MAP_FILE_SUFFIX))
        {
            continue;
        }

        let metadata = match load_metadata(path.as_path()).await {
            Ok(meta) => meta,
            Err(err) => return Err(LoadingError::Map { source: err }),
        };
        map_entries.push(MapEntry::new(path.into(), metadata));
    }

    map_entries.sort_by(|a, b| a.metadata().name().cmp(b.metadata().name()));
    Ok(map_entries)
}

fn map_button(commands: &mut GuiCommands, map: MapEntry) -> Entity {
    commands
        .spawn_button(
            OuterStyle {
                size: Size::new(Val::Percent(100.), Val::Percent(8.)),
                margin: UiRect::new(
                    Val::Percent(0.),
                    Val::Percent(0.),
                    Val::Percent(2.),
                    Val::Percent(2.),
                ),
            },
            map.metadata().name(),
        )
        .insert(map)
        .id()
}

fn select_map_system(
    mut next_state: ResMut<NextState<MapState>>,
    mut events: EventReader<SelectMapEvent>,
) {
    // Exhaust the iterator.
    if events.iter().count() == 0 {
        return;
    }
    next_state.set(MapState::On);
}
