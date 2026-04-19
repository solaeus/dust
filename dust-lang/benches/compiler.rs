#![allow(clippy::disallowed_methods)]

use std::hint::black_box;

use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use dust_lang::{
    compiler::Compiler,
    source::{Source, SourceCode},
};

const SOURCE: &[u8] = br#"
    fn main() {
        let x = 1 + 2;
        let y = x * 3;
        print(y);
    }
"#;

fn compile_bench(source: Source) {
    Compiler::new(source).compile(None).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("compiler");

    let mut source = Source::new();

    source.add_code(SourceCode::borrowed("test", SOURCE));

    group.throughput(Throughput::Elements(1000));
    group.bench_function("compile", |b| {
        b.iter_batched(
            || source.clone(),
            |source| compile_bench(black_box(source)),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
