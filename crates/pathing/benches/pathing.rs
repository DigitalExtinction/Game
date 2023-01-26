use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use bevy::prelude::Transform;
use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
    Throughput,
};
use de_core::objects::{ActiveObjectType, BuildingType, ObjectType};
use de_map::size::MapBounds;
use de_objects::{Ichnography, IchnographyCache};
use de_pathing::{create_finder, PathQueryProps, PathTarget};
use glam::Vec2;
use parry2d::{math::Point, shape::ConvexPolygon};

const MAP_SIZE: f32 = 8000.;

struct IchnographyCacheMock {
    fixed: Ichnography,
}

impl Default for IchnographyCacheMock {
    fn default() -> Self {
        Self {
            fixed: Ichnography::from(
                ConvexPolygon::from_convex_hull(&[
                    Point::new(-8., 8.),
                    Point::new(-8., -8.),
                    Point::new(8., -8.),
                    Point::new(8., 8.),
                ])
                .unwrap(),
            ),
        }
    }
}

impl IchnographyCache for &IchnographyCacheMock {
    fn get_ichnography(&self, _object_type: ObjectType) -> &Ichnography {
        &self.fixed
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
        points.push(MAP_SIZE * Vec2::new(x, y));
    }
    points
}

fn load_entities(number: u32) -> Vec<(Transform, ObjectType)> {
    load_points(number)
        .iter()
        .map(|p| {
            (
                Transform::from_xyz(p.x, 0., -p.y),
                ObjectType::Active(ActiveObjectType::Building(BuildingType::Base)),
            )
        })
        .collect()
}

fn create_finder_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("create_finder");
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    let cache = IchnographyCacheMock::default();

    for num_entities in [100, 1000, 10_000, 100_000] {
        let entities = load_entities(num_entities);

        let bounds = MapBounds::new(Vec2::splat(MAP_SIZE));

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::from_parameter(num_entities), |b| {
            b.iter(|| {
                create_finder(&cache, bounds, entities.clone());
            });
        });
    }
}

fn find_path_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_path");
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    let cache = IchnographyCacheMock::default();

    let points = load_points(100_000);
    let mut index = 0;

    for num_entities in [100, 1000, 10_000, 100_000] {
        let bounds = MapBounds::new(Vec2::splat(MAP_SIZE));
        let finder = create_finder(&cache, bounds, load_entities(num_entities));

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::from_parameter(num_entities), |b| {
            b.iter(|| {
                let start = points[index];
                index = (index + 1) % points.len();
                let target = points[index];
                index = (index + 1) % points.len();
                finder.find_path(
                    start,
                    PathTarget::new(target, PathQueryProps::new(0., 10.), false),
                );
            });
        });
    }
}

criterion_group!(benches, create_finder_benchmark, find_path_benchmark);
criterion_main!(benches);
