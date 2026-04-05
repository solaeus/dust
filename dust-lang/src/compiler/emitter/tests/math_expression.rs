use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn add() {
    let prototype = emit_function("fn foo() -> i32 { let a: i32 = 5; let b: i32 = 3; a - b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::subtract(
                    0,
                    OperandType::I_32,
                    MemoryKind::ENCODED,
                    5,
                    MemoryKind::ENCODED,
                    3
                ),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn multiply() {
    let prototype = emit_function("fn foo() -> i32 { let a: i32 = 3; let b: i32 = 4; a * b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::multiply(
                    0,
                    OperandType::I_32,
                    MemoryKind::ENCODED,
                    3,
                    MemoryKind::ENCODED,
                    4
                ),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn tail() {
    let prototype = emit_function("fn foo() -> i32 { let a: i32 = 10; let b: i32 = 3; a / b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::divide(
                    0,
                    OperandType::I_32,
                    MemoryKind::ENCODED,
                    10,
                    MemoryKind::ENCODED,
                    3
                ),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn modulo() {
    let prototype = emit_function("fn foo() -> i32 { let a: i32 = 10; let b: i32 = 3; a % b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::modulo(
                    0,
                    OperandType::I_32,
                    MemoryKind::ENCODED,
                    10,
                    MemoryKind::ENCODED,
                    3
                ),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn power() {
    let prototype = emit_function("fn foo() -> i32 { let a: i32 = 2; let b: i32 = 3; a ^ b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::power(
                    0,
                    OperandType::I_32,
                    MemoryKind::ENCODED,
                    2,
                    MemoryKind::ENCODED,
                    3
                ),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}
