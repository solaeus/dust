use std::{fmt::Write, hint::black_box, sync::Arc};

use criterion::{Criterion, criterion_group, criterion_main};
use dust_compiler::{
    compiler::Compiler,
    dust_value::DustValue,
    program::Program,
    source::{Code, Source},
};
use dust_vm::{Vm, VmConfig};

const SIEVE_COUNT: usize = 500;

fn create_source() -> String {
    let mut source = String::new();
    let mut count = 0;

    while count < SIEVE_COUNT {
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

    source
}

fn vm_bench(program: Arc<Program>) {
    let vm = Vm::new(
        program,
        VmConfig {
            minimum_object_heap: 0,
            minimum_object_sweep: 0,
        },
    );
    let return_value = vm.run().unwrap().unwrap();

    assert_eq!(return_value, DustValue::I32(42));
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm");

    let mut source = Source::new("bench".to_string());
    let content = create_source();
    let name = format!("{SIEVE_COUNT}_sieves.ds");

    source.add_code(Code::validated(&name, &content));

    let compiler = Compiler::new(source);
    let program = compiler.compile().unwrap();
    let program = Arc::new(program);

    group.bench_function(name, |b| {
        b.iter(|| vm_bench(black_box(Arc::clone(&program))));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
