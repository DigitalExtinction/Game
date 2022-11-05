use bevy::prelude::*;
use de_camera::{CameraDistance, DistanceLabels};
use de_core::stages::GameStage;
use de_terrain::CircleMarker;

use crate::{DISTANCE_FLAG_BIT, MAX_VISIBILITY_DISTANCE};

pub(crate) struct MarkersPlugin;

impl Plugin for MarkersPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            GameStage::PostUpdate,
            update_distance.after(DistanceLabels::Update),
        );
    }
}

fn update_distance(
    mut markers: Query<(&CameraDistance, &mut CircleMarker), Changed<CameraDistance>>,
) {
    for (cam_distance, mut marker) in markers.iter_mut() {
        marker.visibility_mut().update_invisible(
            DISTANCE_FLAG_BIT,
            cam_distance.distance() > MAX_VISIBILITY_DISTANCE,
        );
    }
}
