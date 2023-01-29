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
use iyes_loopless::prelude::*;
use thiserror::Error;

use crate::{menu::Menu, MenuState};

pub(crate) struct MapSelectionPlugin;

impl Plugin for MapSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SelectMapEvent>()
            .add_event::<MapSelectedEvent>()
            .add_enter_system(MenuState::MapSelection, setup)
            .add_exit_system(MenuState::MapSelection, cleanup)
            .add_system_set(
                SystemSet::new()
                    .with_system(init_buttons.run_in_state(MenuState::MapSelection))
                    .with_system(button_system.run_in_state(MenuState::MapSelection))
                    .with_system(select_map_system.run_in_state(AppState::InMenu)),
            );
    }
}

/// When this event is received, menu state is set to map selection.
pub(crate) struct SelectMapEvent {
    next_state: MenuState,
}

impl SelectMapEvent {
    /// # Arguments
    ///
    /// * `next_state` - after a map is selected, menu state is switched to
    ///   this state.
    pub(crate) fn new(next_state: MenuState) -> Self {
        Self { next_state }
    }

    fn next_state(&self) -> MenuState {
        self.next_state
    }
}

/// This event is sent after a map is selected, just before menu state is
/// switched to a next state.
pub(crate) struct MapSelectedEvent {
    path: PathBuf,
}

impl MapSelectedEvent {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Path to the map on the local file system.
    pub(crate) fn path(&self) -> &Path {
        self.path.as_path()
    }
}

#[derive(Resource)]
struct AfterSelectionState(MenuState);

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
}

fn init_buttons(mut commands: GuiCommands, menu: Res<Menu>, task: Option<ResMut<LoadingTask>>) {
    let Some(mut task) = task else { return };
    let Some(result) = future::block_on(future::poll_once(&mut task.0)) else {
        return
    };

    let map_entries = match result {
        Ok(entries) => entries,
        Err(error) => {
            log_full_error!(error);
            panic!("{}", error);
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
    commands.entity(menu.root_node()).add_child(column_node);

    for map in map_entries {
        let button = map_button(&mut commands, map);
        commands.entity(column_node).add_child(button);
    }
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<LoadingTask>();
    commands.remove_resource::<AfterSelectionState>();
}

fn button_system(
    mut commands: Commands,
    interactions: Query<(&Interaction, &MapEntry), Changed<Interaction>>,
    next_state: Res<AfterSelectionState>,
    mut events: EventWriter<MapSelectedEvent>,
) {
    for (&interaction, map) in interactions.iter() {
        if let Interaction::Clicked = interaction {
            commands.insert_resource(NextState(next_state.0));
            events.send(MapSelectedEvent::new(map.path().into()));
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

fn select_map_system(mut commands: Commands, mut events: EventReader<SelectMapEvent>) {
    let Some(event) = events.iter().last() else { return };
    commands.insert_resource(AfterSelectionState(event.next_state()));
    commands.insert_resource(NextState(MenuState::MapSelection));
}
