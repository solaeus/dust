use std::time::Duration;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dust_lang::run;

const SOURCE: &str = r"
    fn fib (n: int) -> int {
        if n <= 0 {
            0
        } else if n == 1 {
            1
        } else {
            fib(n - 1) + fib(n - 2)
        }
    }

    fib(25)
";

fn addictive_addition(source: &str) {
    run(source).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");

    group.measurement_time(Duration::from_secs(15));
    group.bench_function("fibonacci", |b| {
        b.iter(|| addictive_addition(black_box(SOURCE)))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
