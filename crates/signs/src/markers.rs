use bevy::prelude::*;
use de_camera::{CameraDistance, DistanceSet};
use de_core::{baseset::GameSet, state::AppState};
use de_terrain::MarkerVisibility;

use crate::{DISTANCE_FLAG_BIT, MAX_VISIBILITY_DISTANCE};

pub(crate) struct MarkersPlugin;

impl Plugin for MarkersPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            update_distance
                .in_base_set(GameSet::PostUpdate)
                .run_if(in_state(AppState::InGame))
                .after(DistanceSet::Update),
        );
    }
}

fn update_distance(
    mut markers: Query<(&CameraDistance, &mut MarkerVisibility), Changed<CameraDistance>>,
) {
    for (cam_distance, mut visibility) in markers.iter_mut() {
        visibility.0.update_invisible(
            DISTANCE_FLAG_BIT,
            cam_distance.distance() > MAX_VISIBILITY_DISTANCE,
        );
    }
}
