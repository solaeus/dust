use std::time::Duration;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dust_lang::compile;

const LOOP: &str = r"
let mut i = 0

while i < 5_000_000 {
    i += 1
}
";

const FUNCTION: &str = r"
fn addictive_addition() {
    let mut i = 0

    while i < 5_000_000 {
        i += 1
    }
}
";

fn compile_bench(source: &str) {
    compile(source).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut source = String::new();
    let mut group = c.benchmark_group("compile");

    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    for _ in 0..1000 {
        source.push_str(LOOP);
        source.push('\n');
    }

    group.bench_function("compile 1,000 loops", |b| {
        b.iter(|| compile_bench(black_box(&source)))
    });

    for _ in 0..4000 {
        source.push_str(LOOP);
        source.push('\n');
    }

    group.bench_function("compile 5,000 loops", |b| {
        b.iter(|| compile_bench(black_box(&source)))
    });

    source.clear();

    for _ in 0..1000 {
        source.push_str(FUNCTION);
        source.push('\n');
    }

    group.bench_function("compile 1,000 functions", |b| {
        b.iter(|| compile_bench(black_box(&source)))
    });

    for _ in 0..9000 {
        source.push_str(FUNCTION);
        source.push('\n');
    }

    group.bench_function("compile 10,000 functions", |b| {
        b.iter(|| compile_bench(black_box(&source)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
