use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn i8() {
    let prototype = emit_function("fn foo() -> i8 { 42 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_8, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_8],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn u8() {
    let prototype = emit_function("fn foo() -> u8 { 42 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::U_8, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::U_8],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn i16() {
    let prototype = emit_function("fn foo() -> i16 { 42 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_16, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_16],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn u16() {
    let prototype = emit_function("fn foo() -> u16 { 42 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::U_16, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::U_16],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn i32() {
    let prototype = emit_function("fn foo() -> i32 { 42 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn u32() {
    let prototype = emit_function("fn foo() -> u32 { 42 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::U_32, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::U_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn i64() {
    let prototype = emit_function("fn foo() -> i64 { 42 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_64, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_64],
            register_count: 2,
            argument_count: 0,
        }
    );
}

#[test]
fn u64() {
    let prototype = emit_function("fn foo() -> u64 { 42 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::U_64, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::U_64],
            register_count: 2,
            argument_count: 0,
        }
    );
}

#[test]
fn i128() {
    let prototype = emit_function("fn foo() -> i128 { 42 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_128, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_128],
            register_count: 4,
            argument_count: 0,
        }
    );
}

#[test]
fn u128() {
    let prototype = emit_function("fn foo() -> u128 { 42 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::U_128, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::U_128],
            register_count: 4,
            argument_count: 0,
        }
    );
}

#[test]
fn isize() {
    let prototype = emit_function("fn foo() -> isize { 42 }");

    #[cfg(target_pointer_width = "64")]
    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_64, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_64],
            register_count: 2,
            argument_count: 0,
        }
    );

    #[cfg(target_pointer_width = "32")]
    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn usize() {
    let prototype = emit_function("fn foo() -> usize { 42 }");

    #[cfg(target_pointer_width = "64")]
    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::U_64, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::U_64],
            register_count: 2,
            argument_count: 0,
        }
    );

    #[cfg(target_pointer_width = "32")]
    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::U_32, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::U_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}
