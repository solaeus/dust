use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use dust_lang::{lexer::Lexer, parser::Parser, source::SourceFileId};

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

fn parse_bench(source: &[u8]) {
    let lexer = Lexer::new(source);
    let parser = Parser::new(SourceFileId(0), lexer);

    parser.parse_main();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut source = String::new();
    let mut group = c.benchmark_group("parse");

    for _ in 0..10_000 {
        source.push_str(LOOP);
        source.push('\n');
    }

    group.throughput(criterion::Throughput::Bytes(source.len() as u64));
    group.bench_function("parse 10,000 loops", |b| {
        b.iter(|| parse_bench(black_box(source.as_bytes())))
    });

    for _ in 0..40_000 {
        source.push_str(LOOP);
        source.push('\n');
    }

    group.throughput(criterion::Throughput::Bytes(source.len() as u64));
    group.bench_function("parse 50,000 loops", |b| {
        b.iter(|| parse_bench(black_box(source.as_bytes())))
    });

    source.clear();

    for _ in 0..10_000 {
        source.push_str(FUNCTION);
        source.push('\n');
    }

    group.throughput(criterion::Throughput::Bytes(source.len() as u64));
    group.bench_function("parse 10,000 functions", |b| {
        b.iter(|| parse_bench(black_box(source.as_bytes())))
    });

    for _ in 0..90_000 {
        source.push_str(FUNCTION);
        source.push('\n');
    }

    group.throughput(criterion::Throughput::Bytes(source.len() as u64));
    group.bench_function("parse 100,000 functions", |b| {
        b.iter(|| parse_bench(black_box(source.as_bytes())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
