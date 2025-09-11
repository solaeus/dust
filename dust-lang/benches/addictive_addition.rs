use std::{hint::black_box, time::Duration};

use criterion::{Criterion, criterion_group, criterion_main};
use dust_lang::jit_vm::run_main;

const SOURCE: &str = r"
let mut i = 0;

while i < 10_000_000 {
    i += 1;
}
";

fn addictive_addition(source: &str) {
    run_main("addictive_addition".to_string(), source).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("addictive_addition");

    group.measurement_time(Duration::from_secs(15));
    group.bench_function("addictive_addition", |b| {
        b.iter(|| addictive_addition(black_box(SOURCE)))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
