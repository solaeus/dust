#![feature(iterator_try_collect)]

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use dust_lang::Lexer;

fn tokenize(source: &[u8]) {
    for result in Lexer::new(source) {
        result.unwrap();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize");
    let all_ascii = Vec::from_iter(0u8..=127);

    group.bench_function("all ascii bytes repeated 10,000 times", |b| {
        b.iter(|| tokenize(black_box(&all_ascii.repeat(10_000))))
    });

    group.bench_function("all ascii bytes repeated 100,000 times", |b| {
        b.iter(|| tokenize(black_box(&all_ascii.repeat(100_000))))
    });

    let utf8_range = 0..=0x10FFFF;
    let surrogate_range = 0xD800..=0xDFFF;
    let mut bytes = [0u8; 4];
    let mut all_valid_utf8 = Vec::new();

    for codepoint in utf8_range {
        if surrogate_range.contains(&codepoint) {
            continue;
        }

        let character = std::char::from_u32(codepoint).unwrap();

        character.encode_utf8(&mut bytes);
        all_valid_utf8.extend_from_slice(&bytes[..character.len_utf8()]);
    }

    group.bench_function("all utf8 sequences", |b| {
        b.iter(|| tokenize(black_box(&all_valid_utf8)))
    });

    group.bench_function("all utf8 sequences repeated 5 times", |b| {
        b.iter(|| tokenize(black_box(&all_valid_utf8.repeat(5))))
    });

    let mixed_bytes = all_ascii
        .iter()
        .chain(all_valid_utf8.iter())
        .cloned()
        .collect::<Vec<u8>>();

    group.bench_function("all ascii and utf8 bytes", |b| {
        b.iter(|| tokenize(black_box(&mixed_bytes)))
    });

    group.bench_function("all ascii and utf8 bytes repeated 5 times", |b| {
        b.iter(|| tokenize(black_box(&mixed_bytes.repeat(5))))
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
