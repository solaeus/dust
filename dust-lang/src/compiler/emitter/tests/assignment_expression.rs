use smallvec::smallvec;

use crate::{
    compiler::emitter::tests::emit_function,
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn reassignment() {
    let prototype = emit_function("fn foo() -> i32 { let mut x: i32 = 1; x = 2; x }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 1),
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
fn compound_addition() {
    let prototype = emit_function("fn foo() -> i32 { let mut x: i32 = 1; x += 2; x }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 1),
                Instruction::add(
                    0,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::ENCODED,
                    2
                ),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn compound_subtraction() {
    let prototype = emit_function("fn foo() -> i32 { let mut x: i32 = 10; x -= 3; x }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 10),
                Instruction::subtract(
                    0,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::ENCODED,
                    3
                ),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn compound_multiplication() {
    let prototype = emit_function("fn foo() -> i32 { let mut x: i32 = 3; x *= 4; x }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 3),
                Instruction::multiply(
                    0,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::ENCODED,
                    4
                ),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn compound_division() {
    let prototype = emit_function("fn foo() -> i32 { let mut x: i32 = 12; x /= 3; x }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 12),
                Instruction::divide(
                    0,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::ENCODED,
                    3
                ),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn compound_modulo() {
    let prototype = emit_function("fn foo() -> i32 { let mut x: i32 = 10; x %= 3; x }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 10),
                Instruction::modulo(
                    0,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::ENCODED,
                    3
                ),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn compound_power() {
    let prototype = emit_function("fn foo() -> i32 { let mut x: i32 = 2; x ^= 3; x }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 2),
                Instruction::power(
                    0,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::ENCODED,
                    3
                ),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn array_index_assignment_constant() {
    let prototype = emit_function("fn foo() { let mut arr: [i32; 3] = [0, 0, 0]; arr[1] = 42; }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 0),
                Instruction::r#move(1, OperandType::I_32, MemoryKind::ENCODED, 0),
                Instruction::r#move(2, OperandType::I_32, MemoryKind::ENCODED, 0),
                Instruction::r#move(1, OperandType::I_32, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: smallvec![],
            register_count: 3,
            argument_count: 0,
        }
    );
}

#[test]
fn array_index_assignment_dynamic() {
    let prototype =
        emit_function("fn foo(i: i32) { let mut arr: [i32; 3] = [0, 0, 0]; arr[i] = 42; }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(1, OperandType::I_32, MemoryKind::ENCODED, 0),
                Instruction::r#move(2, OperandType::I_32, MemoryKind::ENCODED, 0),
                Instruction::r#move(3, OperandType::I_32, MemoryKind::ENCODED, 0),
                Instruction::check_index(MemoryKind::REGISTER, 0, 3),
                Instruction::set_index(
                    1,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::ENCODED,
                    42
                ),
                Instruction::r#return(),
            ],
            return_types: smallvec![],
            register_count: 4,
            argument_count: 1,
        }
    );
}
