use bevy::prelude::*;
use de_core::{
    baseset::GameSet,
    objects::{MovableSolid, StaticSolid},
    state::AppState,
};

pub(crate) struct DistancePlugin;

impl Plugin for DistancePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            init::<MovableSolid>
                .in_base_set(GameSet::PostUpdate)
                .run_if(in_state(AppState::InGame)),
        )
        .add_system(
            init::<StaticSolid>
                .in_base_set(GameSet::PostUpdate)
                .run_if(in_state(AppState::InGame)),
        )
        .add_system(
            update
                .in_base_set(GameSet::PostUpdate)
                .run_if(in_state(AppState::InGame))
                .in_set(DistanceSet::Update),
        );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemSet)]
pub enum DistanceSet {
    Update,
}

#[derive(Component)]
pub struct CameraDistance(f32);

impl CameraDistance {
    /// Returns the distance between the object and the camera.
    pub fn distance(&self) -> f32 {
        self.0
    }
}

fn init<T: Component>(
    mut commands: Commands,
    camera: Query<&Transform, With<Camera3d>>,
    objects: Query<(Entity, &Transform), Added<T>>,
) {
    let Ok(cam_transform) = camera.get_single() else {
        return;
    };

    for (entity, transform) in objects.iter() {
        commands.entity(entity).insert(CameraDistance(
            cam_transform.translation.distance(transform.translation),
        ));
    }
}

fn update(
    camera: Query<&Transform, With<Camera3d>>,
    mut objects: Query<(&Transform, &mut CameraDistance)>,
) {
    let Ok(cam_transform) = camera.get_single() else {
        return;
    };

    for (transform, mut camera_distance) in objects.iter_mut() {
        let distance = cam_transform.translation.distance(transform.translation);

        // Do not unnecessarily trigger change detection.
        if camera_distance.0 != distance {
            camera_distance.0 = distance;
        }
    }
}
