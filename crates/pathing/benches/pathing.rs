use bevy::prelude::Transform;
use criterion::{
    criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion, PlotConfiguration,
    Throughput,
};
use de_map::size::MapBounds;
use de_objects::Ichnography;
use de_pathing::{create_finder, ExclusionArea, PathQueryProps, PathTarget};
use de_test_utils::{load_points, NumPoints};
use glam::Vec2;
use parry2d::{math::Point, shape::ConvexPolygon};

const MAP_HALF_SIZE: f32 = 4000.;

fn load_exclusions(number: &NumPoints) -> Vec<ExclusionArea> {
    let ichnography = Ichnography::from(
        ConvexPolygon::from_convex_hull(&[
            Point::new(-8., 8.),
            Point::new(-8., -8.),
            Point::new(8., -8.),
            Point::new(8., 8.),
        ])
        .unwrap(),
    );

    load_points(number, MAP_HALF_SIZE - 20.)
        .iter()
        .map(|p| ExclusionArea::from_ichnography(&Transform::from_xyz(p.x, 0., -p.y), &ichnography))
        .collect()
}

fn create_finder_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("create_finder");
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    for number in [
        NumPoints::OneHundred,
        NumPoints::OneThousand,
        NumPoints::TenThousand,
    ] {
        let exclusions = load_exclusions(&number);

        let bounds = MapBounds::new(Vec2::splat(2. * MAP_HALF_SIZE));

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::from_parameter(usize::from(number)), |b| {
            b.iter(|| {
                create_finder(bounds, exclusions.clone());
            });
        });
    }
}

fn find_path_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_path");
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    let points = load_points(&NumPoints::OneHundredThousand, MAP_HALF_SIZE);
    let mut index = 0;

    for number in [
        NumPoints::OneHundred,
        NumPoints::OneThousand,
        NumPoints::TenThousand,
    ] {
        let bounds = MapBounds::new(Vec2::splat(2. * MAP_HALF_SIZE));
        let finder = create_finder(bounds, load_exclusions(&number));

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::from_parameter(usize::from(number)), |b| {
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
