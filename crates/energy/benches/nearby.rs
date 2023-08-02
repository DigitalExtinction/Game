use bevy::prelude::{
    App, Changed, Component, Entity, Query, ResMut, Resource, Time, Timer, Transform, Update, World,
};
use bevy::time::{TimePlugin, TimerMode};
use criterion::{criterion_group, criterion_main, Criterion};
use de_energy::{update_nearby_recv, EnergyReceiver, NearbyUnits};
use de_index::{EntityIndex, LocalCollider};
use de_objects::ObjectCollider;
use parry3d::math::{Isometry, Vector};
use parry3d::shape::{Cuboid, TriMesh};

fn init_world_with_entities(world: &mut World, num_entities: usize) {
    let mut index = EntityIndex::new();

    for _ in 0..num_entities {
        let x: f32 = fastrand::i32(-500..500) as f32;
        let y: f32 = fastrand::i32(-500..500) as f32;

        let collider = LocalCollider::new(
            ObjectCollider::from(TriMesh::from(Cuboid::new(Vector::new(3., 3., 4.)))),
            Isometry::new(Vector::new(x, 0., y), Vector::identity()),
        );

        let entity = world
            .spawn((
                Transform::from_xyz(x, y, 0.0),
                EnergyReceiver,
                NearbyUnits::default(),
            ))
            .id();

        index.insert(entity, collider);
    }
    world.insert_resource(index);
}

fn update_index(
    mut index: ResMut<EntityIndex>,
    moved: Query<(Entity, &Transform), (Changed<Transform>)>,
) {
    for (entity, transform) in moved.iter() {
        let position = Isometry::new(
            transform.translation.into(),
            transform.rotation.to_scaled_axis().into(),
        );
        index.update(entity, position);
    }
}

fn teleport_entities_randomly(mut query: Query<&mut Transform>) {
    for mut transform in query.iter_mut() {
        // we just move the entities somewhere in a random direction
        let offset_x: f32 = fastrand::i32(1..3) as f32;
        let offset_y: f32 = fastrand::i32(1..3) as f32;

        transform.translation.x += offset_x;
        transform.translation.y += offset_y;
    }
}

// moving but deciding a direction and traviling for one second before deciding a new direction
#[derive(Component)]
struct Direction {
    x: f32,
    y: f32,
}

fn init_world_with_entities_moving(world: &mut World, num_entities: usize) {
    let mut index = EntityIndex::new();

    for _ in 0..num_entities {
        let x: f32 = fastrand::i32(-500..500) as f32;
        let y: f32 = fastrand::i32(-500..500) as f32;

        let collider = LocalCollider::new(
            ObjectCollider::from(TriMesh::from(Cuboid::new(Vector::new(3., 3., 4.)))),
            Isometry::new(Vector::new(x, 0., y), Vector::identity()),
        );

        let entity = world
            .spawn((
                Transform::from_xyz(x, y, 0.0),
                EnergyReceiver,
                NearbyUnits::default(),
                Direction { x: 0., y: 0. },
            ))
            .id();

        index.insert(entity, collider);
    }
    world.insert_resource(index);
}

#[derive(Resource)]
struct DirectionTimer(Timer);

fn move_entities_randomly_direction(
    mut query: Query<(&mut Transform, &mut Direction)>,
    mut timer: ResMut<DirectionTimer>,
    mut time: ResMut<Time>,
) {
    for (mut transform, mut direction) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            direction.x = fastrand::i32(-1..2) as f32;
            direction.y = fastrand::i32(-1..2) as f32;
        }

        transform.translation.x += direction.x*time.delta_seconds();
        transform.translation.y += direction.y*time.delta_seconds();
    }
}

fn nearby_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("nearby entity movement scenarios");

    for i in [100, 1_000, 10_000, 100_000].iter() {
        let mut app = App::default();
        init_world_with_entities(&mut app.world, *i);
        app.add_systems(
            Update,
            (teleport_entities_randomly, update_nearby_recv, update_index),
        );

        group.throughput(criterion::Throughput::Elements(*i as u64));
        group.bench_function(format!("{} entities all teleporting randomly", i), |b| {
            b.iter(|| {
                app.update();
            })
        });
    }

    for i in [100, 1_000, 10_000, 100_000].iter() {
        let mut app = App::default();
        init_world_with_entities_moving(&mut app.world, *i);
        app.add_systems(
            Update,
            (
                move_entities_randomly_direction,
                update_nearby_recv,
                update_index,
            ),
        );
        app.add_plugins(TimePlugin);
        app.insert_resource(DirectionTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )));

        group.throughput(criterion::Throughput::Elements(*i as u64));
        group.bench_function(format!("{} entities all moving randomly", i), |b| {
            b.iter(|| {
                app.update();
            })
        });
    }
}

criterion_group!(benches, nearby_benchmark);

criterion_main!(benches);
