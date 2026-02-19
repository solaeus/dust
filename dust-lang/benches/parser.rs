use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use dust_lang::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::SourceFileId,
};

const BENCHES: [(&str, usize); 3] = [
    ("tiny source", 10),
    ("large source", 1_000),
    ("insane source", 50_000),
];

const SOURCE: [u8; 100] = *b"
fn foobar_foobar_foobar() {
    let mut i = 0;

    while i < 5_000_000 {
        i += 666;
    }
}";

fn parse_bench(source: &[u8]) {
    let ParseResult { errors, .. } =
        Parser::new(SourceFileId::MAIN, Lexer::from_bytes(source)).parse();

    assert!(errors.is_empty(), "{errors:#?}");
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");

    for (name, count) in BENCHES.iter() {
        let source = SOURCE.repeat(*count);
        let bytes = SOURCE.len() * count;
        let kilobytes = bytes as f64 / 1000.0;

        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_function(
            format!("{name}: {count} functions, {kilobytes:.2} KB"),
            |b| b.iter(|| parse_bench(black_box(&source))),
        );
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
