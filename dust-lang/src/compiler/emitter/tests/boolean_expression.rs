use smallvec::smallvec;

use crate::{
    compiler::emitter::tests::emit_function,
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn r#true() {
    let prototype = emit_function("fn foo() -> bool { true }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::BOOLEAN, MemoryKind::ENCODED, 1),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::BOOLEAN],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn r#false() {
    let prototype = emit_function("fn foo() -> bool { false }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::BOOLEAN, MemoryKind::ENCODED, 0),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::BOOLEAN],
            register_count: 1,
            argument_count: 0,
        }
    );
}
