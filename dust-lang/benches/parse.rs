use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use dust_lang::{lexer::Lexer, parser::Parser, source::SourceFileId};

const LOOP: &[u8] = b"
let mut i = 0;

while i < 5_000_000 {
    i += 1;
}
";

const FUNCTION: &[u8] = b"
fn() {
    let mut i = 0;

    while i < 5_000_000 {
        i += 1;
    }
};
";

fn parse_bench(source: &[u8]) {
    let lexer = Lexer::from_bytes(source);
    let parser = Parser::new(SourceFileId::MAIN, lexer);

    parser.parse_main();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut source = Vec::new();
    let mut group = c.benchmark_group("parse");

    for _ in 0..10_000 {
        source.extend_from_slice(LOOP);
        source.push(b'\n');
    }

    group.throughput(criterion::Throughput::Bytes(source.len() as u64));
    group.bench_function("parse 10,000 loops", |b| {
        b.iter(|| parse_bench(black_box(&source)))
    });

    for _ in 0..40_000 {
        source.extend_from_slice(LOOP);
        source.push(b'\n');
    }

    group.throughput(criterion::Throughput::Bytes(source.len() as u64));
    group.bench_function("parse 50,000 loops", |b| {
        b.iter(|| parse_bench(black_box(&source)))
    });

    source.clear();

    for _ in 0..10_000 {
        source.extend_from_slice(FUNCTION);
        source.push(b'\n');
    }

    group.throughput(criterion::Throughput::Bytes(source.len() as u64));
    group.bench_function("parse 10,000 functions", |b| {
        b.iter(|| parse_bench(black_box(&source)))
    });

    for _ in 0..90_000 {
        source.extend_from_slice(FUNCTION);
        source.push(b'\n');
    }

    group.throughput(criterion::Throughput::Bytes(source.len() as u64));
    group.bench_function("parse 100,000 functions", |b| {
        b.iter(|| parse_bench(black_box(&source)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
