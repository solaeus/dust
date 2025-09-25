use std::{hint::black_box, time::Duration};

use criterion::{Criterion, criterion_group, criterion_main};
use dust_lang::{Lexer, token::Token};

fn tokenize(source: &[u8]) {
    let _ = Lexer::new(source).unwrap().collect::<Vec<Token>>();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize");
    let ten_tokens = b"D U S T ! ";
    let ten_million_tokens = ten_tokens.repeat(1_000_000);

    group.measurement_time(Duration::from_secs(15));
    group.bench_function("500,000 tokens in 1,000,000 bytes", |b| {
        b.iter(|| tokenize(black_box(&ten_million_tokens)))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
