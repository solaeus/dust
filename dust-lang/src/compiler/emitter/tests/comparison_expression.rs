use smallvec::smallvec;

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
            return_types: smallvec![OperandType::BOOLEAN],
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
            return_types: smallvec![OperandType::BOOLEAN],
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
            return_types: smallvec![OperandType::BOOLEAN],
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
            return_types: smallvec![OperandType::BOOLEAN],
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
            return_types: smallvec![OperandType::BOOLEAN],
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
            return_types: smallvec![OperandType::BOOLEAN],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn runtime_equal() {
    let prototype = emit_function("fn foo(a: i32, b: i32) -> bool { a == b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::equal(
                    true,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::REGISTER,
                    1
                ),
                Instruction::move_with_jump(
                    0,
                    OperandType::BOOLEAN,
                    MemoryKind::ENCODED,
                    0,
                    1,
                    true
                ),
                Instruction::r#move(0, OperandType::BOOLEAN, MemoryKind::ENCODED, 1),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::BOOLEAN],
            register_count: 2,
            argument_count: 2,
        }
    );
}

#[test]
fn runtime_not_equal() {
    let prototype = emit_function("fn foo(a: i32, b: i32) -> bool { a != b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::equal(
                    false,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::REGISTER,
                    1
                ),
                Instruction::move_with_jump(
                    0,
                    OperandType::BOOLEAN,
                    MemoryKind::ENCODED,
                    0,
                    1,
                    true
                ),
                Instruction::r#move(0, OperandType::BOOLEAN, MemoryKind::ENCODED, 1),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::BOOLEAN],
            register_count: 2,
            argument_count: 2,
        }
    );
}

#[test]
fn runtime_less_than() {
    let prototype = emit_function("fn foo(a: i32, b: i32) -> bool { a < b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::less(
                    true,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::REGISTER,
                    1
                ),
                Instruction::move_with_jump(
                    0,
                    OperandType::BOOLEAN,
                    MemoryKind::ENCODED,
                    0,
                    1,
                    true
                ),
                Instruction::r#move(0, OperandType::BOOLEAN, MemoryKind::ENCODED, 1),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::BOOLEAN],
            register_count: 2,
            argument_count: 2,
        }
    );
}

#[test]
fn runtime_less_than_or_equal() {
    let prototype = emit_function("fn foo(a: i32, b: i32) -> bool { a <= b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::less_equal(
                    true,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::REGISTER,
                    1
                ),
                Instruction::move_with_jump(
                    0,
                    OperandType::BOOLEAN,
                    MemoryKind::ENCODED,
                    0,
                    1,
                    true
                ),
                Instruction::r#move(0, OperandType::BOOLEAN, MemoryKind::ENCODED, 1),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::BOOLEAN],
            register_count: 2,
            argument_count: 2,
        }
    );
}
