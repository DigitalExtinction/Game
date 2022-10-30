use std::marker::PhantomData;

use bevy::prelude::*;
use de_core::{
    objects::MovableSolid,
    projection::{ToFlat, ToMsl},
    stages::GameStage,
    state::GameState,
};
use de_map::size::MapBounds;
use de_objects::EXCLUSION_OFFSET;
use iyes_loopless::prelude::*;

pub(crate) struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            GameStage::PreMovement,
            setup_entities.run_in_state(GameState::Playing),
        )
        .add_system_set_to_stage(
            GameStage::Movement,
            SystemSet::new().with_system(
                update_transform
                    .run_in_state(GameState::Playing)
                    .label(MovementLabels::UpdateTransform),
            ),
        );
    }
}

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
pub(crate) enum MovementLabels {
    UpdateTransform,
}

/// Velocity is computed in stages, this is a generic over all of them.
#[derive(Component)]
pub(crate) struct DesiredVelocity<T> {
    velocity: Vec2,
    // PhantomData<fn() -> T> gives this safe Send/Sync impls
    _m: PhantomData<fn() -> T>,
}

impl<T> DesiredVelocity<T> {
    pub(crate) fn velocity(&self) -> Vec2 {
        self.velocity
    }

    /// Returns true if the velocity is zero.
    pub(crate) fn stationary(&self) -> bool {
        self.velocity == Vec2::ZERO
    }

    pub(crate) fn stop(&mut self) {
        self.velocity = Vec2::ZERO;
    }

    pub(crate) fn update(&mut self, velocity: Vec2) {
        self.velocity = velocity;
    }
}

impl<T> Default for DesiredVelocity<T> {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            _m: PhantomData,
        }
    }
}

/// Real velocity as applied to transform of the each movable object.
#[derive(Component, Default)]
pub(crate) struct ObjectVelocity {
    /// Velocity during the last update.
    previous: Vec3,
    /// Current velocity.
    current: Vec3,
    heading: f32,
    heading_changed: bool,
}

impl ObjectVelocity {
    pub(crate) fn update(&mut self, velocity: Vec3, heading: f32) {
        self.previous = self.current;
        self.current = velocity;
        self.heading_changed = self.heading != heading;
        self.heading = heading;
    }

    /// Returns mean velocity over the last frame duration.
    fn frame(&self) -> Vec3 {
        self.current.lerp(self.previous, 0.5)
    }

    fn heading(&self) -> f32 {
        self.heading
    }

    fn heading_changed(&self) -> bool {
        self.heading_changed
    }
}

pub(crate) fn add_desired_velocity<T: 'static>(
    mut commands: Commands,
    objects: Query<Entity, (With<MovableSolid>, Without<DesiredVelocity<T>>)>,
) {
    for entity in objects.iter() {
        commands
            .entity(entity)
            .insert(DesiredVelocity::<T>::default());
    }
}

fn setup_entities(
    mut commands: Commands,
    objects: Query<Entity, (With<MovableSolid>, Without<ObjectVelocity>)>,
) {
    for entity in objects.iter() {
        commands.entity(entity).insert(ObjectVelocity::default());
    }
}

fn update_transform(
    time: Res<Time>,
    bounds: Res<MapBounds>,
    mut objects: Query<(&ObjectVelocity, &mut Transform)>,
) {
    let time_delta = time.delta_seconds();
    for (velocity, mut transform) in objects.iter_mut() {
        let frame_velocity = velocity.frame();

        // Do not trigger Bevy's change detection when not necessary.
        if frame_velocity != Vec3::ZERO {
            transform.translation = clamp(
                bounds.as_ref(),
                transform.translation + time_delta * frame_velocity,
            );
        }

        if velocity.heading_changed() {
            transform.rotation = Quat::from_rotation_y(velocity.heading());
        }
    }
}

fn clamp(bounds: &MapBounds, translation: Vec3) -> Vec3 {
    let offset = Vec2::splat(EXCLUSION_OFFSET);
    let min = bounds.min() + offset;
    let max = bounds.max() - offset;
    let clipped = translation.to_flat().clamp(min, max).to_msl();
    Vec3::new(clipped.x, translation.y, clipped.z)
}
