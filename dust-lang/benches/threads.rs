use std::time::Duration;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dust_lang::run;

const SOURCE: &str = r#"
    let mut i = 0

    while i < 1_000 {
        i += 1

        spawn(
            fn () { random_int(0, 10); }
        )
    }
"#;

fn threads(source: &str) {
    run(source).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("threads");

    group.measurement_time(Duration::from_secs(15));
    group.bench_function("threads", |b| b.iter(|| threads(black_box(SOURCE))));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
