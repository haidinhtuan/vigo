//! Benchmarks for Vigo Vietnamese input engine.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vigo::{Engine, InputMethod, transform_buffer};

fn bench_transform_simple(c: &mut Criterion) {
    c.bench_function("transform_simple", |b| {
        b.iter(|| transform_buffer(black_box("vieetj")))
    });
}

fn bench_transform_sentence(c: &mut Criterion) {
    let input = "xin chaof cacs banj tooi laf nguwowif Vieetj Nam";
    c.bench_function("transform_sentence", |b| {
        b.iter(|| transform_buffer(black_box(input)))
    });
}

fn bench_engine_feed(c: &mut Criterion) {
    c.bench_function("engine_feed", |b| {
        let mut engine = Engine::telex();
        b.iter(|| {
            engine.clear();
            for ch in "vieetj".chars() {
                black_box(engine.feed(ch));
            }
        })
    });
}

fn bench_engine_sentence(c: &mut Criterion) {
    let input = "xin chaof cacs banj";
    c.bench_function("engine_sentence", |b| {
        let mut engine = Engine::telex();
        b.iter(|| {
            engine.clear();
            for ch in input.chars() {
                black_box(engine.feed(ch));
            }
        })
    });
}

fn bench_vni(c: &mut Criterion) {
    c.bench_function("vni_simple", |b| {
        let mut engine = Engine::vni();
        b.iter(|| {
            engine.clear();
            for ch in "vie6t5".chars() {
                black_box(engine.feed(ch));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_transform_simple,
    bench_transform_sentence,
    bench_engine_feed,
    bench_engine_sentence,
    bench_vni,
);
criterion_main!(benches);
