use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use bevy::prelude::GlobalTransform;
use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
    Throughput,
};
use de_index::Ichnography;
use de_map::size::MapBounds;
use de_pathing::create_finder;
use glam::Vec2;
use parry2d::{math::Point, shape::ConvexPolygon};

const MAP_SIZE: f32 = 8000.;

fn load_points(number: u32) -> Vec<Vec2> {
    let mut points_path: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    points_path.push("test_data");
    points_path.push(format!("{}-points.txt", number));
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

fn load_entities(number: u32) -> Vec<(GlobalTransform, Ichnography)> {
    load_points(number)
        .iter()
        .map(|p| {
            let ichnography = Ichnography::new(
                ConvexPolygon::from_convex_hull(&[
                    Point::new(p.x - 10., p.y + 10.),
                    Point::new(p.x - 10., p.y - 10.),
                    Point::new(p.x + 10., p.y - 10.),
                    Point::new(p.x + 10., p.y + 10.),
                ])
                .unwrap(),
            );
            (GlobalTransform::identity(), ichnography)
        })
        .collect()
}

fn create_finder_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("create_finder");
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    for num_entities in [100, 1000, 10_000, 100_000] {
        let entities = load_entities(num_entities);

        let bounds = MapBounds::new(Vec2::splat(MAP_SIZE));

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::from_parameter(num_entities), |b| {
            b.iter(|| {
                create_finder(bounds, entities.clone());
            });
        });
    }
}

criterion_group!(benches, create_finder_benchmark);
criterion_main!(benches);
