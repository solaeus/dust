use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use dust_lang::jit_vm::run_main;

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

fn fibonacci(source: &str) {
    run_main(source).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");

    group.bench_function("fibonacci", |b| b.iter(|| fibonacci(black_box(SOURCE))));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
