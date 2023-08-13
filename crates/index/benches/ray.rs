use bevy::prelude::*;
use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
    Throughput,
};
use de_index::{EntityIndex, LocalCollider, SpatialQuery};
use de_objects::ObjectCollider;
use de_test_utils::load_points;
use glam::Vec2;
use parry3d::{
    math::{Isometry, Point, Vector},
    query::Ray,
    shape::{Cuboid, TriMesh},
};

const MAP_SIZE: f32 = 2000.;

#[derive(Resource)]
struct Rays {
    rays: Vec<Ray>,
    index: usize,
}

impl Rays {
    fn new() -> Self {
        Self {
            rays: Vec::new(),
            index: 0,
        }
    }

    fn insert(&mut self, ray: Ray) {
        self.rays.push(ray);
    }
}

#[derive(Resource)]
struct MaxDistance(f32);

impl Iterator for Rays {
    type Item = Ray;

    fn next(&mut self) -> Option<Ray> {
        if self.index >= self.rays.len() {
            self.index = 0;
            return None;
        }

        let next = self.rays[self.index];
        self.index += 1;
        Some(next)
    }
}

fn setup_world(world: &mut World, num_entities: u32, max_distance: f32) {
    let points = load_points(num_entities.try_into().unwrap(), MAP_SIZE);
    let mut index = EntityIndex::new();

    for (i, point) in points.iter().enumerate() {
        let collider = LocalCollider::new(
            ObjectCollider::from(TriMesh::from(Cuboid::new(Vector::new(3., 3., 4.)))),
            Isometry::new(Vector::new(point.x, 0., point.y), Vector::identity()),
        );
        index.insert(Entity::from_raw(i as u32), collider);
    }

    let mut rays = Rays::new();
    let ray_origins = load_points(1000.try_into().unwrap(), MAP_SIZE);
    let ray_dirs = load_points(1000.try_into().unwrap(), MAP_SIZE);
    for (origin, dir) in ray_origins.iter().zip(ray_dirs.iter()) {
        let dir = if dir.length() < 0.0001 {
            Vec2::new(1., 0.)
        } else {
            dir.normalize()
        };

        rays.insert(Ray::new(
            Point::new(origin.x, 0., origin.y),
            Vector::new(dir.x, 0., dir.y),
        ));
    }

    world.insert_resource(index);
    world.insert_resource(rays);
    world.insert_resource(MaxDistance(max_distance));
}

fn cast_ray(mut rays: ResMut<Rays>, max_distance: Res<MaxDistance>, index: SpatialQuery<()>) {
    for ray in rays.as_mut() {
        index.cast_ray(&ray, max_distance.0, None);
    }
}

fn ray_cast_benchmark(c: &mut Criterion) {
    for max_distance in [0.1, 1., 10., 100., f32::INFINITY] {
        let mut group = c.benchmark_group(format!(
            "Ray Cast - Small Entities - Max Distance {max_distance}m"
        ));

        let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
        group.plot_config(plot_config);

        for num_entities in [100, 1000, 10_000, 100_000] {
            let mut app = App::new();
            app.add_systems(Update, cast_ray);
            setup_world(&mut app.world, num_entities, max_distance);

            group.throughput(Throughput::Elements(1));
            group.bench_function(BenchmarkId::from_parameter(num_entities), |b| {
                b.iter(|| app.update());
            });
        }

        group.finish();
    }
}

criterion_group!(benches, ray_cast_benchmark);
criterion_main!(benches);
