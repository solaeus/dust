use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn return_value() {
    let prototype = emit_function("fn bar() -> i32 { 42 } fn foo() -> i32 { bar() }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::call(0, MemoryKind::ENCODED, 1, u16::MAX),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn multiple_arguments() {
    let prototype =
        emit_function("fn add(a: i32, b: i32) -> i32 { a + b } fn foo() -> i32 { add(1, 2) }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 1),
                Instruction::r#move(1, OperandType::I_32, MemoryKind::ENCODED, 2),
                Instruction::call(0, MemoryKind::ENCODED, 1, 0),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 2,
            argument_count: 0,
        }
    );
}

#[test]
fn nested_call() {
    let prototype = emit_function(
        "fn double(x: i32) -> i32 { x + x } fn triple(x: i32) -> i32 { x + x + x } fn foo(n: i32) -> i32 { double(triple(n)) }",
    );

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(1, OperandType::I_32, MemoryKind::REGISTER, 0),
                Instruction::call(1, MemoryKind::ENCODED, 2, 1),
                Instruction::call(0, MemoryKind::ENCODED, 1, 1),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 2,
            argument_count: 1,
        }
    );
}
