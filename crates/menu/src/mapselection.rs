use std::path::{Path, PathBuf};

use async_std::{fs, io, stream::StreamExt};
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use de_core::{
    assets::asset_path,
    gconfig::GameConfig,
    log_full_error,
    player::Player,
    state::{AppState, GameState, MenuState},
};
use de_map::{
    io::{load_metadata, MapLoadingError, MAP_FILE_SUFFIX},
    meta::MapMetadata,
};
use futures_lite::future;
use iyes_loopless::prelude::*;
use thiserror::Error;

use crate::menu::{despawn_root_nodes, Text};

pub(crate) struct MapSelectionPlugin;

impl Plugin for MapSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(MenuState::MapSelection, setup)
            .add_exit_system(MenuState::MapSelection, cleanup)
            .add_exit_system(MenuState::MapSelection, despawn_root_nodes)
            .add_system_set(
                SystemSet::new()
                    .with_system(init_buttons.run_in_state(MenuState::MapSelection))
                    .with_system(button_system.run_in_state(MenuState::MapSelection)),
            );
    }
}

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

fn init_buttons(mut commands: Commands, text: Res<Text>, task: Option<ResMut<LoadingTask>>) {
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

    let root_node = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                size: Size::new(Val::Percent(25.), Val::Percent(100.)),
                margin: UiRect::all(Val::Auto),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: Color::rgba(0., 0., 0., 0.).into(),
            ..default()
        })
        .id();

    for map in map_entries {
        let button = map_button(&mut commands, text.as_ref(), map);
        commands.entity(root_node).add_child(button);
    }
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<LoadingTask>();
}

fn button_system(
    mut commands: Commands,
    interactions: Query<(&Interaction, &MapEntry), Changed<Interaction>>,
) {
    for (&interaction, map) in interactions.iter() {
        if let Interaction::Clicked = interaction {
            commands.insert_resource(GameConfig::new(map.path(), Player::Player1));
            commands.insert_resource(NextState(MenuState::None));
            commands.insert_resource(NextState(AppState::InGame));
            commands.insert_resource(NextState(GameState::Loading));
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

    map_entries.sort_by(|a, b| b.metadata().name().cmp(a.metadata().name()));
    Ok(map_entries)
}

fn map_button(commands: &mut Commands, text: &Text, map: MapEntry) -> Entity {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(8.)),
                margin: UiRect::new(
                    Val::Percent(0.),
                    Val::Percent(0.),
                    Val::Percent(2.),
                    Val::Percent(2.),
                ),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                map.metadata().name(),
                text.button_text_style(),
            ));
        })
        .insert(map)
        .id()
}
