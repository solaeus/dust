use smallvec::smallvec;

use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn one_argument() {
    let prototype =
        emit_function("fn foo() -> i32 { fn add_one(x: i32) -> i32 { x + 1 } add_one(5) }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 5),
                Instruction::call(0, MemoryKind::ENCODED, 1, 0),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn no_arguments() {
    let prototype =
        emit_function("fn foo() -> i32 { fn returns_five() -> i32 { 5 } returns_five() }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::call(0, MemoryKind::ENCODED, 1, u16::MAX),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn multiple_arguments() {
    let prototype =
        emit_function("fn foo() -> i32 { fn add(a: i32, b: i32) -> i32 { a + b } add(3, 4) }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 3),
                Instruction::r#move(1, OperandType::I_32, MemoryKind::ENCODED, 4),
                Instruction::call(0, MemoryKind::ENCODED, 1, 0),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 2,
            argument_count: 0,
        }
    );
}

#[test]
fn parameter_passthrough() {
    let prototype =
        emit_function("fn foo(x: i32) -> i32 { fn double(n: i32) -> i32 { n + n } double(x) }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(1, OperandType::I_32, MemoryKind::REGISTER, 0),
                Instruction::call(0, MemoryKind::ENCODED, 1, 1),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 2,
            argument_count: 1,
        }
    );
}

#[test]
fn generic_monomorphization() {
    let prototype = emit_function(
        "fn identity<T>(x: T) -> T { x } fn foo(n: i32) -> i32 { identity::<i32>(n) }",
    );

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(1, OperandType::I_32, MemoryKind::REGISTER, 0),
                Instruction::call(0, MemoryKind::ENCODED, 1, 1),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 2,
            argument_count: 1,
        }
    );
}
