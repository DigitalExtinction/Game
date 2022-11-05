use bevy::prelude::*;
use de_core::{
    objects::{MovableSolid, StaticSolid},
    stages::GameStage,
    state::AppState,
};
use iyes_loopless::prelude::*;

pub(crate) struct DistancePlugin;

impl Plugin for DistancePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            GameStage::PostUpdate,
            SystemSet::new()
                .with_system(init::<MovableSolid>.run_in_state(AppState::InGame))
                .with_system(init::<StaticSolid>.run_in_state(AppState::InGame))
                .with_system(
                    update
                        .run_in_state(AppState::InGame)
                        .label(DistanceLabels::Update),
                ),
        );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub enum DistanceLabels {
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
    let Ok(cam_transform) = camera.get_single() else { return };

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
    let Ok(cam_transform) = camera.get_single() else { return };

    for (transform, mut camera_distance) in objects.iter_mut() {
        let distance = cam_transform.translation.distance(transform.translation);

        // Do not unnecessarily trigger change detection.
        if camera_distance.0 != distance {
            camera_distance.0 = distance;
        }
    }
}
