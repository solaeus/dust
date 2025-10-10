#![feature(iterator_try_collect)]

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use dust_lang::lexer::Lexer;

fn tokenize(source: &[u8]) {
    for result in Lexer::new(source) {
        result.unwrap();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize");
    let all_ascii = Vec::from_iter(0u8..=127);

    let all_ascii_10k = all_ascii.repeat(10_000);
    group.throughput(Throughput::Bytes(all_ascii_10k.len() as u64));
    group.bench_function("combined: ASCII x10k", |b| {
        b.iter(|| tokenize(black_box(&all_ascii_10k)))
    });

    let all_ascii_100k = all_ascii.repeat(100_000);
    group.throughput(Throughput::Bytes(all_ascii_100k.len() as u64));
    group.bench_function("combined: ASCII x100k", |b| {
        b.iter(|| tokenize(black_box(&all_ascii_100k)))
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

    group.throughput(Throughput::Bytes(all_valid_utf8.len() as u64));
    group.bench_function("combined: UTF-8 all", |b| {
        b.iter(|| tokenize(black_box(&all_valid_utf8)))
    });

    let all_valid_utf8_5x = all_valid_utf8.repeat(5);
    group.throughput(Throughput::Bytes(all_valid_utf8_5x.len() as u64));
    group.bench_function("combined: UTF-8 all x5", |b| {
        b.iter(|| tokenize(black_box(&all_valid_utf8_5x)))
    });

    let mixed_bytes = all_ascii
        .iter()
        .chain(all_valid_utf8.iter())
        .cloned()
        .collect::<Vec<u8>>();

    group.throughput(Throughput::Bytes(mixed_bytes.len() as u64));
    group.bench_function("combined: Mixed x1", |b| {
        b.iter(|| tokenize(black_box(&mixed_bytes)))
    });

    let mixed_bytes_5x = mixed_bytes.repeat(5);
    group.throughput(Throughput::Bytes(mixed_bytes_5x.len() as u64));
    group.bench_function("combined: Mixed x5", |b| {
        b.iter(|| tokenize(black_box(&mixed_bytes_5x)))
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
