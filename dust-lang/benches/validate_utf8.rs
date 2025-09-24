use std::{hint::black_box, time::Duration};

use criterion::{Criterion, criterion_group, criterion_main};
use dust_lang::validate_utf8_and_find_token_starts;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("validate_utf8");

    group.measurement_time(Duration::from_secs(15));

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

    group.bench_function("all_valid_utf8", |b| {
        b.iter(|| validate_utf8_and_find_token_starts(black_box(&all_valid_utf8)))
    });

    let all_ascii = Vec::from_iter(0u8..=127);
    let big_ascii = all_ascii.repeat(1_000_000);

    group.bench_function("big_ascii", |b| {
        b.iter(|| validate_utf8_and_find_token_starts(black_box(&big_ascii)))
    });

    let bigger_ascii = all_ascii.repeat(10_000_000);

    group.bench_function("bigger_ascii", |b| {
        b.iter(|| validate_utf8_and_find_token_starts(black_box(&bigger_ascii)))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
