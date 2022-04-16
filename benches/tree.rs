use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration, Throughput,
};
use de::game::tree::{Disc, Rectangle, Tree};
use glam::Vec2;

fn load_points(number: u32, suffix: &str) -> Vec<Vec2> {
    let mut points_path: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    points_path.push("test_data");
    points_path.push(format!("{}-{}.txt", number, suffix));
    let reader = BufReader::new(File::open(points_path).unwrap());

    let mut points = Vec::with_capacity(number as usize);
    for line in reader.lines() {
        let line = line.unwrap();
        let mut numbers = line.split_whitespace();
        let x: f32 = numbers.next().unwrap().parse().unwrap();
        let y: f32 = numbers.next().unwrap().parse().unwrap();
        points.push(Vec2::new(x, y));
    }
    points
}

struct Discs {
    discs: Vec<Disc>,
    index: usize,
}

impl Discs {
    fn load(radius: f32) -> Self {
        let points = load_points(1000, "disc-points");
        let mut discs = Vec::with_capacity(1000);
        for point in points {
            discs.push(Disc::new(point, radius));
        }
        Self { discs, index: 0 }
    }
}

impl Iterator for Discs {
    type Item = Disc;

    fn next(&mut self) -> Option<Self::Item> {
        let result = Some(self.discs[self.index]);
        self.index = (self.index + 1).div_euclid(self.discs.len());
        result
    }
}

fn init_tree(num_points: u32) -> Tree<[u32; 5]> {
    let points = load_points(num_points, "points");
    let mut tree = Tree::with_capacity(num_points as usize, Rectangle::new(Vec2::ZERO, Vec2::ONE));
    for (i, &point) in points.iter().enumerate() {
        tree.insert([i as u32; 5], point);
    }
    tree
}

fn tree_within_benchmark(c: &mut Criterion) {
    for radius in [0.01, 0.05] {
        let mut group = c.benchmark_group(format!("Tree Elements Within {}", radius));

        let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
        group.plot_config(plot_config);

        for num_elements in [100, 1000, 10_000, 100_000] {
            let tree = init_tree(num_elements);
            let mut discs = Discs::load(radius);

            group.throughput(Throughput::Elements(1));
            group.bench_function(BenchmarkId::from_parameter(num_elements), |b| {
                b.iter(|| black_box(tree.within_disc(black_box(discs.next().unwrap()))));
            });
        }
        group.finish();
    }
}

criterion_group!(benches, tree_within_benchmark);
criterion_main!(benches);
