use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use dust_compiler::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
};

const SOURCE: &[u8] = include_bytes!("../../examples/eratosthenes_sieve.ds");

const BENCHES: [(&str, usize); 3] = [
    ("tiny source", 1),
    ("large source", 100),
    ("insane source", 5_000),
];

fn parse_bench(source: &[u8]) {
    let ParseResult { errors, .. } = Parser::new_standalone(Lexer::unvalidated(source)).parse();

    assert!(errors.is_empty(), "{errors:#?}");
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");

    for (name, count) in BENCHES.iter() {
        let source = SOURCE.repeat(*count);

        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_function(name.to_string(), |b| {
            b.iter(|| parse_bench(black_box(&source)))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
