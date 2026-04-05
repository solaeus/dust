use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn equal() {
    let prototype = emit_function("fn foo() -> bool { let a: i32 = 1; let b: i32 = 2; a == b }");

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
fn less_than() {
    let prototype = emit_function("fn foo() -> bool { let a: i32 = 1; let b: i32 = 2; a < b }");

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
fn not_equal() {
    let prototype = emit_function("fn foo() -> bool { let a: i32 = 1; let b: i32 = 2; a != b }");

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
fn greater_than() {
    let prototype = emit_function("fn foo() -> bool { let a: i32 = 2; let b: i32 = 1; a > b }");

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
fn less_than_or_equal() {
    let prototype = emit_function("fn foo() -> bool { let a: i32 = 1; let b: i32 = 2; a <= b }");

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
fn greater_than_or_equal() {
    let prototype = emit_function("fn foo() -> bool { let a: i32 = 2; let b: i32 = 1; a >= b }");

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
