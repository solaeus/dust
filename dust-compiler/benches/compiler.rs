#![expect(clippy::disallowed_methods)]

use std::{fmt::Write, hint::black_box};

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use dust_compiler::{
    compiler::Compiler,
    source::{Code, Source},
};

const SIEVE_COUNTS: [usize; 3] = [10, 100, 1000];

fn create_source(sieve_count: usize) -> Vec<u8> {
    let mut source = String::new();

    let mut count = 0;

    while count < sieve_count {
        let _ = write!(
            &mut source,
            "
fn eratosthenes_sieve_{count}() -> i32 {{
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
        "
        );

        count += 1;
    }

    source.push_str("fn main() -> i32 {\n");

    for i in 0..sieve_count {
        let _ = writeln!(&mut source, "eratosthenes_sieve_{}();", i);
    }

    source.push_str("42\n}\n");

    source.into_bytes()
}

fn compiler_bench(content: &[u8]) {
    let mut source = Source::new("bench".to_string());

    source.add_code(Code::unvalidated("eratosthenes_sieve.ds", content));

    Compiler::new(source).compile().unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("compiler");

    for count in SIEVE_COUNTS {
        let source = create_source(count);

        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_function(format!("{count}_sieves"), |b| {
            b.iter_with_large_drop(|| compiler_bench(black_box(&source)))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
