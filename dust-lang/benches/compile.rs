use std::{hint::black_box, time::Duration};

use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use dust_lang::compiler::compile_main;

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

fn compile_bench(source: String) {
    compile_main(source).unwrap();
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

    group.throughput(Throughput::Elements(1000));
    group.bench_function("compile 1,000 loops", |b| {
        b.iter_batched(
            || source.clone(),
            |input: String| compile_bench(black_box(input)),
            BatchSize::SmallInput,
        )
    });

    for _ in 0..4000 {
        source.push_str(LOOP);
        source.push('\n');
    }

    group.throughput(Throughput::Elements(5000));
    group.bench_function("compile 5,000 loops", |b| {
        b.iter_batched(
            || source.clone(),
            |input: String| compile_bench(black_box(input)),
            BatchSize::SmallInput,
        )
    });

    source.clear();

    for _ in 0..1000 {
        source.push_str(FUNCTION);
        source.push('\n');
    }

    group.throughput(Throughput::Elements(1000));
    group.bench_function("compile 1,000 functions", |b| {
        b.iter_batched(
            || source.clone(),
            |input: String| compile_bench(black_box(input)),
            BatchSize::SmallInput,
        )
    });

    for _ in 0..9000 {
        source.push_str(FUNCTION);
        source.push('\n');
    }

    group.throughput(Throughput::Elements(10_000));
    group.bench_function("compile 10,000 functions", |b| {
        b.iter_batched(
            || source.clone(),
            |input: String| compile_bench(black_box(input)),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
