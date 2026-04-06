use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn variable() {
    let prototype = emit_function("fn foo() -> i32 { let x: i32 = 5; -x }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::negate(0, OperandType::I_32, MemoryKind::ENCODED, 5),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn constant() {
    let prototype = emit_function("fn foo() -> i32 { -5 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 65531),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn parameter() {
    let prototype = emit_function("fn foo(x: i32) -> i32 { -x }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::negate(0, OperandType::I_32, MemoryKind::REGISTER, 0),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 1,
        }
    );
}
