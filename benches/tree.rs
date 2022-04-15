use criterion::{criterion_group, criterion_main, Criterion};
use de::game::tree::{Disc, Rectangle, Tree};
use glam::Vec2;
use rand::prelude::*;

fn tree_within_benchmark(c: &mut Criterion) {
    c.bench_function("Tree Elements Within", |b| {
        let num_elements = 10000;
        let max_num_elements = num_elements + 5000;

        let tree_max = Vec2::new(1000., 1000.);
        let mut tree = Tree::with_capacity(num_elements, Rectangle::new(Vec2::ZERO, tree_max));
        let mut rng = ThreadRng::default();
        let mut to_delete = Vec::new();

        for i in 0..max_num_elements {
            let rand_x: f32 = rng.gen();
            let rand_y: f32 = rng.gen();
            let random_point = Vec2::new(rand_x * tree_max.x, rand_y * tree_max.y);
            to_delete.push(tree.insert((i, i, i), random_point));
        }

        for _ in 0..(max_num_elements - num_elements) {
            let index = rng.gen_range(0..to_delete.len());
            let last_index = to_delete.len() - 1;
            to_delete.swap(index, last_index);
            to_delete.pop().unwrap();
        }

        for tree_item in to_delete {
            tree.remove(tree_item);
        }

        let mut discs = Vec::new();
        let mut disc_index = 0;
        for _ in 0..1000 {
            let center = Vec2::new(rng.gen::<f32>() * tree_max.x, rng.gen::<f32>() * tree_max.y);
            let radius: f32 = rng.gen::<f32>() * 50.;
            discs.push(Disc::new(center, radius));
        }

        b.iter(|| {
            tree.within_disc(discs[disc_index]);
            disc_index = (disc_index + 1).div_euclid(discs.len());
        })
    });
}

criterion_group!(benches, tree_within_benchmark);
criterion_main!(benches);
