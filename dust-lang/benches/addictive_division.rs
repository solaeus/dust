use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dust_lang::run;

const SOURCE: &str = r"
    let mut i = 1.7976931348623157e308

    while i > 1.0 {
        i /= 1.00014196662;
    }
";

fn addictive_division(source: &str) {
    run(source).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("addictive_division");

    group.measurement_time(Duration::from_secs(15));
    group.bench_function("addictive_division", |b| {
        b.iter(|| addictive_division(black_box(SOURCE)))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
