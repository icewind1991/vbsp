use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use vbsp::{Bsp, Vector};

const MAP_BYTES: &[u8] = include_bytes!("../koth_bagel_rc2a.bsp");

fn from_bytes(c: &mut Criterion) {
    c.bench_function("parse bsp", |b| {
        b.iter(|| Bsp::read(black_box(MAP_BYTES)).unwrap())
    });
}

fn leaf_at(c: &mut Criterion) {
    let bsp = Bsp::read(MAP_BYTES).unwrap();

    c.bench_function("get leaf at position", |b| {
        b.iter(|| {
            black_box(bsp.leaf_at(black_box(Vector {
                x: 0.,
                y: 0.,
                z: 0.,
            })))
        });
    });
}

criterion_group!(benches, leaf_at, from_bytes);
criterion_main!(benches);
