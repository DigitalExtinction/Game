use bevy::ecs::prelude::*;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

const MOVE_MARGIN_PX: f32 = 40.0;

#[derive(Debug)]
enum HorizonalMovement {
    None,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Component)]
pub struct Movement {
    horizontal: HorizonalMovement,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            horizontal: HorizonalMovement::None,
        }
    }
}

pub fn setup(mut commands: Commands) {
    commands.spawn().insert(Movement::default());
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 5.0, 2.0).looking_at(Vec3::ZERO, -Vec3::Z),
        ..Default::default()
    });
}

pub fn mouse_movement(
    mut event_reader: EventReader<CursorMoved>,
    windows: Res<Windows>,
    mut query: Query<&mut Movement>,
) {
    if let Some(event) = event_reader.iter().last() {
        let x = event.position.x;
        let y = event.position.y;

        let window = windows.get_primary().unwrap();
        let width = window.width();
        let height = window.height();

        let mut movement = query.single_mut();
        if x < MOVE_MARGIN_PX {
            movement.horizontal = HorizonalMovement::Left;
        } else if x > (width - MOVE_MARGIN_PX) {
            movement.horizontal = HorizonalMovement::Right;
        } else if y < MOVE_MARGIN_PX {
            movement.horizontal = HorizonalMovement::Up;
        } else if y > (height - MOVE_MARGIN_PX) {
            movement.horizontal = HorizonalMovement::Down;
        } else {
            movement.horizontal = HorizonalMovement::None;
        }
    }
}

pub fn move_horizontaly(query: Query<&Movement>, mut camera: Query<&mut Transform, With<Camera>>) {
    let movement = query.single();
    if let HorizonalMovement::None = movement.horizontal {
        return;
    }

    let mut transform = camera.single_mut();
    let right = transform.local_x();
    let down = transform.local_y();
    match movement.horizontal {
        HorizonalMovement::Left => transform.translation -= right * 0.1,
        HorizonalMovement::Right => transform.translation += right * 0.1,
        HorizonalMovement::Up => transform.translation -= down * 0.1,
        HorizonalMovement::Down => transform.translation += down * 0.1,
        HorizonalMovement::None => (),
    };
}

pub fn zoom(
    mut mouse_wheel: EventReader<MouseWheel>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    let mut transform = camera.single_mut();
    let direction = transform.forward();
    // TODO calculate intersection with terrain and multiply direction by
    // distance from that point

    // TODO limit to some minimum and maximum distance to terrain
    for event in mouse_wheel.iter() {
        transform.translation += (event.y as f32) * direction;
    }
}
