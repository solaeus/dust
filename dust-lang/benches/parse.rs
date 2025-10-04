use std::{hint::black_box, time::Duration};

use criterion::{Criterion, criterion_group, criterion_main};
use dust_lang::parser::parse_main;

const LOOP: &str = r"
let mut i = 0;

while i < 5_000_000 {
    i += 1;
}
";

const FUNCTION: &str = r"
fn() {
    let mut i = 0;

    while i < 5_000_000 {
        i += 1;
    }
};
";

fn parse_bench(source: String) {
    parse_main(source);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut source = String::new();
    let mut group = c.benchmark_group("parse");

    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    for _ in 0..1000 {
        source.push_str(LOOP);
        source.push('\n');
    }

    group.bench_function("parse 1,000 loops", |b| {
        b.iter(|| parse_bench(black_box(source.clone())))
    });

    for _ in 0..4000 {
        source.push_str(LOOP);
        source.push('\n');
    }

    group.bench_function("parse 5,000 loops", |b| {
        b.iter(|| parse_bench(black_box(source.clone())))
    });

    source.clear();

    for _ in 0..1000 {
        source.push_str(FUNCTION);
        source.push('\n');
    }

    group.bench_function("parse 1,000 functions", |b| {
        b.iter(|| parse_bench(black_box(source.clone())))
    });

    for _ in 0..9000 {
        source.push_str(FUNCTION);
        source.push('\n');
    }

    group.bench_function("parse 10,000 functions", |b| {
        b.iter(|| parse_bench(black_box(source.clone())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
