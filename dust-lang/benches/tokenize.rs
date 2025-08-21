use std::{hint::black_box, time::Duration};

use criterion::{Criterion, criterion_group, criterion_main};
use dust_lang::tokenize;

fn tokenize_bench(source: &str) {
    tokenize(source).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize");
    let ten_tokens = "DUSTISGOOD";
    let ten_million_tokens = ten_tokens.repeat(10_000_000);

    group.measurement_time(Duration::from_secs(15));
    group.bench_function("tokenize", |b| {
        b.iter(|| tokenize_bench(black_box(&ten_million_tokens)))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
