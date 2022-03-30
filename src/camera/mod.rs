use crate::states::GameStates;
use bevy::prelude::*;

pub struct MainCameraPlugin;

impl Plugin for MainCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameStates::InGame).with_system(setup));
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 5.0, 0.0).looking_at(Vec3::ZERO, -Vec3::Z),
        ..Default::default()
    });
}
