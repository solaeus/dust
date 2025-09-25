use std::{hint::black_box, time::Duration};

use criterion::{Criterion, criterion_group, criterion_main};
use dust_lang::lexer::validate_utf8_and_find_token_spans;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("validate_utf8");

    group.measurement_time(Duration::from_secs(15));

    let all_ascii = Vec::from_iter(0u8..=127);

    group.bench_function("all ascii bytes repeated 100,000 times", |b| {
        b.iter(|| validate_utf8_and_find_token_spans(black_box(&all_ascii.repeat(100_000))))
    });

    group.bench_function("all ascii bytes repeated 1,000,000 times", |b| {
        b.iter(|| validate_utf8_and_find_token_spans(black_box(&all_ascii.repeat(1_000_000))))
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
        b.iter(|| validate_utf8_and_find_token_spans(black_box(&all_valid_utf8)))
    });

    group.bench_function("all utf8 sequences repeated 10 times", |b| {
        b.iter(|| validate_utf8_and_find_token_spans(black_box(&all_valid_utf8.repeat(10))))
    });

    let mixed_bytes = all_ascii
        .iter()
        .chain(all_valid_utf8.iter())
        .cloned()
        .collect::<Vec<u8>>();

    group.bench_function("all ascii and utf8 bytes", |b| {
        b.iter(|| validate_utf8_and_find_token_spans(black_box(&mixed_bytes)))
    });

    group.bench_function("all ascii and utf8 bytes repeated 10 times", |b| {
        b.iter(|| validate_utf8_and_find_token_spans(black_box(&mixed_bytes.repeat(10))))
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
