use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn and() {
    let prototype =
        emit_function("fn foo() -> bool { let a: bool = true; let b: bool = false; a && b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::test(false, MemoryKind::CONSTANT, 0, 1),
                Instruction::r#move(0, OperandType::BOOLEAN, MemoryKind::CONSTANT, 1),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::BOOLEAN],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn or() {
    let prototype =
        emit_function("fn foo() -> bool { let a: bool = true; let b: bool = false; a || b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::test(true, MemoryKind::CONSTANT, 0, 1),
                Instruction::r#move(0, OperandType::BOOLEAN, MemoryKind::CONSTANT, 1),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::BOOLEAN],
            register_count: 1,
            argument_count: 0,
        }
    );
}
