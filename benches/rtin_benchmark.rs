use criterion::{criterion_group, criterion_main, Criterion};
use de::terrain::grid::{DiscretePoint, ValueGrid};
use de::terrain::rtin::RtinBuilder;
use rand::prelude::*;

pub fn rtin_benchmark(c: &mut Criterion) {
    c.bench_function("RTIN benchmark", |b| {
        let mut rng = rand::thread_rng();
        let mut elevations = ValueGrid::with_zeros(2u16.pow(8) + 1);
        for v in 0..elevations.size() {
            for u in 0..elevations.size() {
                elevations.set_value(
                    DiscretePoint {
                        u: u as u32,
                        v: v as u32,
                    },
                    rng.gen_range(0.0..0.08),
                );
            }
        }

        b.iter(|| RtinBuilder::new(&elevations, 0.1).build())
    });
}

criterion_group!(benches, rtin_benchmark);
criterion_main!(benches);
