use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dust_lang::run;

const SOURCE: &str = r"
    let mut i = 5_000_000

    while i > 0 {
        i -= 1
    }
";

fn addictive_subtraction(source: &str) {
    run(source).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("addictive_subtraction");

    group.measurement_time(Duration::from_secs(15));
    group.bench_function("addictive_subtraction", |b| {
        b.iter(|| addictive_subtraction(black_box(SOURCE)))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
