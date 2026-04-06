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
                Instruction::r#move(0, OperandType::BOOLEAN, MemoryKind::ENCODED, 0),
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
                Instruction::r#move(0, OperandType::BOOLEAN, MemoryKind::ENCODED, 1),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::BOOLEAN],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn runtime_and() {
    let prototype = emit_function("fn foo(a: bool, b: bool) -> bool { a && b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::test(false, MemoryKind::REGISTER, 0, 1),
                Instruction::move_with_jump(
                    0,
                    OperandType::BOOLEAN,
                    MemoryKind::REGISTER,
                    1,
                    1,
                    true
                ),
                Instruction::r#move(0, OperandType::BOOLEAN, MemoryKind::REGISTER, 0),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::BOOLEAN],
            register_count: 2,
            argument_count: 2,
        }
    );
}

#[test]
fn runtime_or() {
    let prototype = emit_function("fn foo(a: bool, b: bool) -> bool { a || b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::test(true, MemoryKind::REGISTER, 0, 1),
                Instruction::move_with_jump(
                    0,
                    OperandType::BOOLEAN,
                    MemoryKind::REGISTER,
                    1,
                    1,
                    true
                ),
                Instruction::r#move(0, OperandType::BOOLEAN, MemoryKind::REGISTER, 0),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::BOOLEAN],
            register_count: 2,
            argument_count: 2,
        }
    );
}
