use smallvec::smallvec;

use crate::{
    compiler::emitter::tests::emit_function,
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn f32() {
    let prototype = emit_function("fn foo() -> f32 { 3.14 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::F_32, MemoryKind::CONSTANT, 0),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::F_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn f64() {
    let prototype = emit_function("fn foo() -> f64 { 2.718 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::F_64, MemoryKind::CONSTANT, 0),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::F_64],
            register_count: 2,
            argument_count: 0,
        }
    );
}
