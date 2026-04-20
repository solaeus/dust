use smallvec::smallvec;

use crate::{
    compiler::emitter::tests::emit_function,
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn literal() {
    let prototype = emit_function("fn foo() -> Range<i32> { 1..10 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 1),
                Instruction::r#move(1, OperandType::I_32, MemoryKind::ENCODED, 10),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32, OperandType::I_32],
            register_count: 2,
            argument_count: 0,
        }
    );
}

#[test]
fn runtime() {
    let prototype = emit_function("fn foo(a: i32, b: i32) -> Range<i32> { a..b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::REGISTER, 0),
                Instruction::r#move(1, OperandType::I_32, MemoryKind::REGISTER, 1),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32, OperandType::I_32],
            register_count: 2,
            argument_count: 2,
        }
    );
}
