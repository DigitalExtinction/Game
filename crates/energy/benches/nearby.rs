use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{
    App, Changed, Component, Entity, Query, Res, ResMut, Schedule, Time, Transform, Update, Vec2,
    World,
};
use bevy::time::TimePlugin;
use criterion::{criterion_group, criterion_main, Criterion};
use de_energy::{update_nearby_recv, EnergyReceiver, NearbyUnits};
use de_index::{EntityIndex, LocalCollider};
use de_objects::ObjectCollider;
use parry3d::math::{Isometry, Vector};
use parry3d::shape::{Cuboid, TriMesh};

const MAP_SIZE: f32 = 2000.;
const DISTANCE_FROM_MAP_EDGE: f32 = 100.;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UpdateUnits;

#[derive(Component)]
struct UnitNumber(u32);

fn update_index(
    mut index: ResMut<EntityIndex>,
    moved: Query<(Entity, &Transform), Changed<Transform>>,
) {
    for (entity, transform) in moved.iter() {
        let position = Isometry::new(
            transform.translation.into(),
            transform.rotation.to_scaled_axis().into(),
        );
        index.update(entity, position);
    }
}

fn load_points(number: u32) -> Vec<Vec2> {
    let mut points_path: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    points_path.push("test_data");
    points_path.push(format!("{number}-points.txt"));
    let reader = BufReader::new(File::open(points_path).unwrap());

    let mut points = Vec::with_capacity(number as usize);
    for line in reader.lines() {
        let line = line.unwrap();
        let mut numbers = line.split_whitespace();
        let x: f32 = numbers.next().unwrap().parse().unwrap();
        let y: f32 = numbers.next().unwrap().parse().unwrap();
        points.push((MAP_SIZE - DISTANCE_FROM_MAP_EDGE) * Vec2::new(x, y));
    }
    points
}

fn init_world_with_entities_moving(world: &mut World, num_entities: u32) {
    let mut index = EntityIndex::new();
    let points = load_points(num_entities);

    for (i, point) in points.into_iter().enumerate() {
        let x: f32 = point.x;
        let y: f32 = point.y;

        let collider = LocalCollider::new(
            ObjectCollider::from(TriMesh::from(Cuboid::new(Vector::new(3., 3., 4.)))),
            Isometry::new(Vector::new(x, 0., y), Vector::identity()),
        );

        let entity = world
            .spawn((
                Transform::from_xyz(x, y, 0.0),
                EnergyReceiver,
                NearbyUnits::default(),
                UnitNumber(i as u32),
            ))
            .id();

        index.insert(entity, collider);
    }

    world.insert_resource(index);
}

/// Move entities in circles of radius N / 2.
fn move_entities_in_circle(time: Res<Time>, mut query: Query<&mut Transform>) {
    for mut transform in query.iter_mut() {
        // Change direction (counter)clockwise based on entity_mum % 2 == 0
        let direction = if transform.translation.x % 2. == 0. {
            1.
        } else {
            -1.
        };

        let t = time.elapsed_seconds();
        let radius = DISTANCE_FROM_MAP_EDGE / 2.;

        let x = radius * (t * 0.5 * direction).sin();
        let y = radius * (t * 0.5 * direction).cos();

        transform.translation.x = x;
        transform.translation.y = y;
    }
}

fn nearby_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("nearby entity movement scenarios");

    for i in [100, 1_000, 10_000].iter() {
        let mut app = App::default();
        init_world_with_entities_moving(&mut app.world, *i);
        app.add_systems(Update, (update_nearby_recv,));
        app.add_plugins(TimePlugin);

        let update_units_schedule = Schedule::default();
        app.add_schedule(UpdateUnits, update_units_schedule);

        app.add_systems(UpdateUnits, (update_index, move_entities_in_circle));

        group.throughput(criterion::Throughput::Elements(*i as u64));
        group.bench_function(format!("{} entities all moving in circles", i), |b| {
            b.iter_custom(|iters| {
                let time = std::time::Instant::now();
                let mut duration_updating_units = std::time::Duration::default();

                for _ in 0..iters {
                    let update_unit_time = std::time::Instant::now();
                    app.world.run_schedule(UpdateUnits);
                    duration_updating_units += update_unit_time.elapsed();

                    app.update();
                }

                time.elapsed() - duration_updating_units
            })
        });
    }
}

criterion_group!(benches, nearby_benchmark);

criterion_main!(benches);
