#![feature(iterator_try_collect, iter_intersperse)]

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use dust_lang::lexer::Lexer;

const SOURCE: &[u8] = br#"
fn fib (n: int) -> int {
    if n <= 0 {
        0
    } else if n == 1 {
        1
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

let mut count = 1;

while count <= 15 {
	if count % 15 == 0 {
		write_line("fizzbuzz");
	} else if count % 3 == 0 {
		write_line("fizz");
	} else if count % 5 == 0 {
		write_line("buzz");
	} else {
	    write_line(count as str);
	}

	count += 1;
}

fn hello_world() {
    write_line("Hello, world!");
    write_line("Enter your name...");

    let name = read_line();

    write_line("Hello " + name + "!");
}

hello_world();
"#;

fn tokenize(source: &[u8]) {
    let mut lexer = Lexer::from_bytes(source);

    for _ in &mut lexer {}

    if let Some(index) = lexer.error_index() {
        panic!(
            "Invalid UTF-8 detected at {index}: \"{}\".",
            String::from_utf8_lossy(&source[index..])
        );
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");
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
    let mut byte_buffer = [0; 4];
    let mut mixed_bytes = Vec::new();

    for codepoint in utf8_range {
        if surrogate_range.contains(&codepoint) {
            continue;
        }

        let ascii = all_ascii.next().or_else(|| all_ascii.next()).unwrap();

        mixed_bytes.push(b' ');
        mixed_bytes.push(ascii);

        let utf8_character = std::char::from_u32(codepoint).unwrap();

        utf8_character.encode_utf8(&mut byte_buffer);
        mixed_bytes.push(b' ');
        mixed_bytes.extend_from_slice(&byte_buffer[..utf8_character.len_utf8()]);
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

    {
        group.throughput(Throughput::Bytes(SOURCE.len() as u64));
        group.bench_function("Small source", |b| b.iter(|| tokenize(black_box(SOURCE))));
    }

    let medium_source = SOURCE.repeat(20);

    {
        group.throughput(Throughput::Bytes(medium_source.len() as u64));
        group.bench_function("Medium source", |b| {
            b.iter(|| tokenize(black_box(&medium_source)))
        });
    }

    let large_source = SOURCE.repeat(100);

    {
        group.throughput(Throughput::Bytes(large_source.len() as u64));
        group.bench_function("Large source", |b| {
            b.iter(|| tokenize(black_box(&large_source)))
        });
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
