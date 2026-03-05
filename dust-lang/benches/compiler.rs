use std::{hint::black_box, time::Duration};

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use dust_lang::prelude::*;

const SOURCE_FILES: &[(&str, &str)] = &[(
    "main.ds",
    r#"
        fn main() {
            let x = 1 + 2;
            let y = x * 3;
            print(y);
        }
    "#,
)];

fn compile_bench(source: &[(&str, &str)]) {
    compile(source).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut source = String::new();
    let mut group = c.benchmark_group("compiler");

    group.throughput(Throughput::Elements(1000));
    group.bench_function("compile", |b| {
        b.iter(|| compile_bench(black_box(SOURCE_FILES)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
