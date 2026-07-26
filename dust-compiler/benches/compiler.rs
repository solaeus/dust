#![expect(clippy::disallowed_methods)]

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use dust_compiler::{
    compiler::Compiler,
    source::{Code, Source},
};

const SOURCE: &[u8] = include_bytes!("../../examples/eratosthenes_sieve.ds");

fn compiler_bench(content: &[u8]) {
    let mut source = Source::new();

    source.add_code(Code::unvalidated("eratosthenes_sieve.ds", content));

    Compiler::new(source)
        .compile("eratosthenes_sieve".to_string())
        .unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("compiler");

    group.throughput(Throughput::Bytes(SOURCE.len() as u64));
    group.bench_function("eratosthenes_sieve.ds", |b| {
        b.iter(|| compiler_bench(black_box(SOURCE)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
