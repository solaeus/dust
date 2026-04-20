use smallvec::smallvec;

use crate::{
    compiler::emitter::tests::emit_function,
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn add() {
    let prototype = emit_function("fn foo() -> i32 { let a: i32 = 5; let b: i32 = 3; a - b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 2),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
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
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 12),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
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
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 3),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
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
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 1),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
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
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 8),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn runtime_add() {
    let prototype = emit_function("fn foo(a: i32, b: i32) -> i32 { a + b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::add(
                    0,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::REGISTER,
                    1
                ),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 2,
            argument_count: 2,
        }
    );
}

#[test]
fn runtime_subtract() {
    let prototype = emit_function("fn foo(a: i32, b: i32) -> i32 { a - b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::subtract(
                    0,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::REGISTER,
                    1
                ),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 2,
            argument_count: 2,
        }
    );
}

#[test]
fn runtime_multiply() {
    let prototype = emit_function("fn foo(a: i32, b: i32) -> i32 { a * b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::multiply(
                    0,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::REGISTER,
                    1
                ),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 2,
            argument_count: 2,
        }
    );
}

#[test]
fn runtime_divide() {
    let prototype = emit_function("fn foo(a: i32, b: i32) -> i32 { a / b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::divide(
                    0,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::REGISTER,
                    1
                ),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 2,
            argument_count: 2,
        }
    );
}

#[test]
fn runtime_modulo() {
    let prototype = emit_function("fn foo(a: i32, b: i32) -> i32 { a % b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::modulo(
                    0,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::REGISTER,
                    1
                ),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 2,
            argument_count: 2,
        }
    );
}

#[test]
fn runtime_power() {
    let prototype = emit_function("fn foo(a: i32, b: i32) -> i32 { a ^ b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::power(
                    0,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::REGISTER,
                    1
                ),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 2,
            argument_count: 2,
        }
    );
}
