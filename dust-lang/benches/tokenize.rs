#![feature(iterator_try_collect, iter_intersperse)]

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

    {
        let all_ascii_10k = all_ascii.repeat(10_000);

        group.throughput(Throughput::Bytes(all_ascii_10k.len() as u64));
        group.bench_function("All ASCII x10k", |b| {
            b.iter(|| tokenize(black_box(&all_ascii_10k)))
        });
    }

    {
        let all_ascii_100k = all_ascii.repeat(100_000);

        group.throughput(Throughput::Bytes(all_ascii_100k.len() as u64));
        group.bench_function("All ASCII x100k", |b| {
            b.iter(|| tokenize(black_box(&all_ascii_100k)))
        });
    }

    let utf8_range = 0..=0x10FFFF;
    let surrogate_range = 0xD800..=0xDFFF;
    let mut bytes = [0u8; 4];
    let mut all_valid_utf8 = Vec::new();

    for codepoint in utf8_range.clone() {
        if surrogate_range.contains(&codepoint) {
            continue;
        }

        let character = std::char::from_u32(codepoint).unwrap();

        character.encode_utf8(&mut bytes);
        all_valid_utf8.extend_from_slice(&bytes[..character.len_utf8()]);
    }

    {
        group.throughput(Throughput::Bytes(all_valid_utf8.len() as u64));
        group.bench_function("Full UTF-8 range", |b| {
            b.iter(|| tokenize(black_box(&all_valid_utf8)))
        });
    }

    {
        let all_valid_utf8_5x = all_valid_utf8.repeat(5);

        group.throughput(Throughput::Bytes(all_valid_utf8_5x.len() as u64));
        group.bench_function("Full UTF-8 range x5", |b| {
            b.iter(|| tokenize(black_box(&all_valid_utf8_5x)))
        });
    }

    let mut all_ascii = all_ascii.into_iter().cycle();
    let mut mixed_bytes = Vec::new();

    for (index, codepoint) in utf8_range.enumerate() {
        if surrogate_range.contains(&codepoint) {
            continue;
        }

        if index % 2 == 0 {
            let ascii = all_ascii.next().or_else(|| all_ascii.next()).unwrap();

            mixed_bytes.push(ascii);

            continue;
        }

        let character = std::char::from_u32(codepoint).unwrap();

        character.encode_utf8(&mut bytes);
        mixed_bytes.extend_from_slice(&bytes[..character.len_utf8()]);
    }

    {
        group.throughput(Throughput::Bytes(mixed_bytes.len() as u64));
        group.bench_function("Mixed ASCII and non-ASCII x1", |b| {
            b.iter(|| tokenize(black_box(&mixed_bytes)))
        });
    }

    {
        let mixed_bytes_5x = mixed_bytes.repeat(5);

        group.throughput(Throughput::Bytes(mixed_bytes_5x.len() as u64));
        group.bench_function("Mixed ASCII and non-ASCII x5", |b| {
            b.iter(|| tokenize(black_box(&mixed_bytes_5x)))
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
