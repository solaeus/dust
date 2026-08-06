#![expect(clippy::disallowed_methods)]

use std::{fmt::Write, hint::black_box};

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use dust_compiler::{
    compiler::Compiler,
    source::{Code, Source},
};

fn create_source() -> Vec<u8> {
    let mut source = String::with_capacity(1024 * 100);
    let mut count = 0;

    while source.len() < 1024 * 100 {
        let _ = write!(
            &mut source,
            "
fn eratosthenes_sieve_{}() -> i32 {{
    let target = 50;
    let mut is_composite = [false; 50];
    let mut factor = 2;

    while factor * factor < target {{
        if !is_composite[factor] {{
            let mut multiple = factor * factor;

            while multiple < target {{
                is_composite[multiple] = true;
                multiple += factor;
            }}
        }}

        factor += 1;
    }}

    let mut count = 0;
    let mut candidate = 2;

    while candidate < target {{
        if !is_composite[candidate] {{
            count += 1;
        }}

        candidate += 1;
    }}

    count
}}
        ",
            count
        );

        count += 1;
    }

    source.push_str("fn main() -> i32 {\n");

    for i in 0..count {
        let _ = writeln!(&mut source, "eratosthenes_sieve_{}();", i);
    }

    source.push_str("42\n}\n");

    source.into_bytes()
}

fn compiler_bench(content: &[u8]) {
    let mut source = Source::new();

    source.add_code(Code::unvalidated("eratosthenes_sieve.ds", content));

    Compiler::new(source)
        .compile("eratosthenes_sieve".to_string())
        .unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("compiler");
    let source = create_source();

    group.throughput(Throughput::Bytes(source.len() as u64));
    group.bench_function("eratosthenes_sieve.ds", |b| {
        b.iter_with_large_drop(|| compiler_bench(black_box(&source)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
