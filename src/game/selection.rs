use super::{collisions::Intersector, objects::Playable, GameStates};
use crate::math::ray::Ray;
use bevy::{
    ecs::system::SystemParam,
    input::{mouse::MouseButtonInput, ElementState},
    prelude::{
        App, Camera, Commands, Component, Entity, EventReader, GlobalTransform, MouseButton,
        Plugin, Query, Res, SystemSet, With,
    },
    window::Windows,
};
use glam::{Vec2, Vec3};
use std::collections::HashSet;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameStates::Playing).with_system(mouse_click_event),
        );
    }
}

#[derive(Component)]
pub struct Selected;

#[derive(SystemParam)]
struct MouseInWorld<'w, 's> {
    windows: Res<'w, Windows>,
    cameras: Query<'w, 's, (&'static GlobalTransform, &'static Camera)>,
}

impl<'w, 's> MouseInWorld<'w, 's> {
    fn mouse_ray(&self) -> Option<Ray> {
        let window = self.windows.get_primary().unwrap();

        // Normalized to values between -1.0 to 1.0 with (0.0, 0.0) in the
        // middle of the screen.
        let cursor_position = match window.cursor_position() {
            Some(position) => {
                let screen_size = Vec2::new(window.width() as f32, window.height() as f32);
                (position / screen_size) * 2.0 - Vec2::ONE
            }
            None => return None,
        };

        let (camera_transform, camera) = self.cameras.single();
        let camera_transform_mat = camera_transform.compute_matrix();
        let camera_projection = camera.projection_matrix;

        let screen_to_world = camera_transform_mat * camera_projection.inverse();
        let world_to_screen = camera_projection * camera_transform_mat;

        // Depth of camera near plane in screen coordinates.
        let near_plane = world_to_screen.transform_point3(-Vec3::Z * camera.near).z;
        let ray_origin = screen_to_world.transform_point3(cursor_position.extend(near_plane));
        let ray_direction = ray_origin - camera_transform.translation;
        Some(Ray::new(ray_origin, ray_direction))
    }
}

#[derive(SystemParam)]
struct Selector<'w, 's> {
    commands: Commands<'w, 's>,
    selected: Query<'w, 's, Entity, With<Selected>>,
}

impl<'w, 's> Selector<'w, 's> {
    fn select_single(&mut self, entity: Option<Entity>) {
        let entities = match entity {
            Some(entity) => vec![entity],
            None => Vec::new(),
        };
        self.select(&entities);
    }

    fn select(&mut self, entities: &[Entity]) {
        let selected: HashSet<Entity> = self.selected.iter().collect();
        let desired: HashSet<Entity> = entities.iter().cloned().collect();

        for deselect in &selected - &desired {
            self.commands.entity(deselect).remove::<Selected>();
        }
        for select in &desired - &selected {
            self.commands.entity(select).insert(Selected);
        }
    }
}

fn mouse_click_event(
    mut event: EventReader<MouseButtonInput>,
    playable: Intersector<With<Playable>>,
    mouse: MouseInWorld,
    mut selector: Selector,
) {
    if !event
        .iter()
        .any(|e| e.button == MouseButton::Left && e.state == ElementState::Pressed)
    {
        return;
    }

    let mouse_ray = match mouse.mouse_ray() {
        Some(ray) => ray,
        None => return,
    };
    selector.select_single(
        playable
            .ray_intersection(&mouse_ray)
            .map(|(entity, _)| entity),
    );
}
