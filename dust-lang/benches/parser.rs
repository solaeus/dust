use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use dust_lang::prelude::*;

const BENCHES: [(&str, usize); 3] = [
    ("tiny source", 1),
    ("large source", 100),
    ("insane source", 5_000),
];

const SOURCE: [u8; 1000] = *br#"
fn foobar_foobar_foobar() {
    let mut i = 0;

    while i < 5_000 {
        i += 42;
    }
}

struct Foo<T> {
    bar: u8,
    baz: i8,
    qux: u16,
    quux: i16,
    quuz: u32,
    corge: i32,
    grault: u64,
    garply: i64,
    waldo: u128,
    fred: i128,
    alice: f32,
    bob: f64,
    wendy: bool,
    xavier: char,
    yvonne: str,
    mallory: [T],
}

enum Color {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Indigo,
    Violet,
}

fn main() {
    let foo = Foo {
        bar: 1,
        baz: -1,
        qux: 2,
        quux: -2,
        quuz: 3,
        corge: -3,
        grault: 4,
        garply: -4,
        waldo: 5,
        fred: -5,
        alice: 3.14,
        bob: 2.71828,
        wendy: true,
        xavier: 'x',
        yvonne: "hello",
        mallory: [1, 2, 3],
    };

    let colors = [
        Color::Red,
        Color::Orange,
        Color::Yellow,
        Color::Green,
        Color::Blue,
        Color::Indigo,
        Color::Violet
    ];
}"#;

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
        group.bench_function(format!("{name}: {kilobytes:.2} KB"), |b| {
            b.iter(|| parse_bench(black_box(&source)))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
