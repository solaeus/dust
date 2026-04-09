use smallvec::smallvec;

use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn variable() {
    let prototype = emit_function("fn foo() -> bool { let x: bool = true; !x }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::negate(0, OperandType::BOOLEAN, MemoryKind::ENCODED, 1),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::BOOLEAN],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn parameter() {
    let prototype = emit_function("fn foo(x: bool) -> bool { !x }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::negate(0, OperandType::BOOLEAN, MemoryKind::REGISTER, 0),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::BOOLEAN],
            register_count: 1,
            argument_count: 1,
        }
    );
}
