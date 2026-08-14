use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Address, Instruction, Memory, OperandType},
    prototype::Prototype,
};

#[test]
fn boolean() {
    assert_program_eq!(
        "fn main() -> bool { true }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::BOOLEAN, Address::new(Memory::ENCODED, true as u16)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::BOOLEAN],
                register_count: 1,
                argument_count: 0,
            },
        ],
        return_type: DustType::Boolean
    );
}

#[test]
fn i8() {
    assert_program_eq!(
        "fn main() -> i8 { -42 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_8, Address::new( Memory::ENCODED, (-42_i16) as u16)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_8],
                register_count: 1,
                argument_count: 0,
            },
        ],
        return_type: DustType::I8
    );
}

#[test]
fn i16() {
    assert_program_eq!(
        "fn main() -> i16 { -42 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_16, Address::new(Memory::ENCODED, (-42_i16) as u16)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_16],
                register_count: 1,
                argument_count: 0,
            },
        ],
        return_type: DustType::I16
    );
}

#[test]
fn i32() {
    assert_program_eq!(
        "fn main() -> i32 { -42 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_32, Address::new(Memory::ENCODED, (-42_i32) as u16)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_32],
                register_count: 1,
                argument_count: 0,
            },
        ],
        return_type: DustType::I32
    );
}

#[test]
fn i64_encodable() {
    assert_program_eq!(
        "fn main() -> i64 { 42 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 42)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 2,
                argument_count: 0,
            },
        ],
        return_type: DustType::I64
    );
}

#[test]
fn i64_not_encodable() {
    assert_program_eq!(
        "fn main() -> i64 { 9223372036854775807 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::CONSTANT, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 2,
                argument_count: 0,
            },
        ],
        return_type: DustType::I64
    );
}

#[test]
fn i128_encodable() {
    assert_program_eq!(
        "fn main() -> i128 { 42 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_128, Address::new(Memory::ENCODED, 42)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_128],
                register_count: 4,
                argument_count: 0,
            },
        ],
        return_type: DustType::I128
    );
}

#[test]
fn i128_not_encodable() {
    assert_program_eq!(
        "fn main() -> i128 { 170141183460469231731687303715884105727 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_128, Address::new(Memory::CONSTANT, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_128],
                register_count: 4,
                argument_count: 0,
            },
        ],
        return_type: DustType::I128
    );
}

#[test]
fn u8() {
    assert_program_eq!(
        "fn main() -> u8 { 42 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::U_8, Address::new(Memory::ENCODED, 42)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::U_8],
                register_count: 1,
                argument_count: 0,
            },
        ],
        return_type: DustType::U8
    );
}

#[test]
fn u16() {
    assert_program_eq!(
        "fn main() -> u16 { 42 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::U_16, Address::new(Memory::ENCODED, 42)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::U_16],
                register_count: 1,
                argument_count: 0,
            },
        ],
        return_type: DustType::U16
    );
}

#[test]
fn u32() {
    assert_program_eq!(
        "fn main() -> u32 { 42 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::U_32, Address::new(Memory::ENCODED, 42)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::U_32],
                register_count: 1,
                argument_count: 0,
            },
        ],
        return_type: DustType::U32
    );
}

#[test]
fn u64_encodable() {
    assert_program_eq!(
        "fn main() -> u64 { 42 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::U_64, Address::new(Memory::ENCODED, 42)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::U_64],
                register_count: 2,
                argument_count: 0,
            },
        ],
        return_type: DustType::U64
    );
}

#[test]
fn u64_not_encodable() {
    assert_program_eq!(
        "fn main() -> u64 { 18446744073709551615 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::U_64, Address::new(Memory::CONSTANT, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::U_64],
                register_count: 2,
                argument_count: 0,
            },
        ],
        return_type: DustType::U64
    );
}

#[test]
fn u128_encodable() {
    assert_program_eq!(
        "fn main() -> u128 { 42 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::U_128, Address::new(Memory::ENCODED, 42)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::U_128],
                register_count: 4,
                argument_count: 0,
            },
        ],
        return_type: DustType::U128
    );
}

#[test]
fn u128_not_encodable() {
    assert_program_eq!(
        "fn main() -> u128 { 18446744073709551615 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::U_128, Address::new(Memory::CONSTANT, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::U_128],
                register_count: 4,
                argument_count: 0,
            },
        ],
        return_type: DustType::U128
    );
}

#[test]
fn f32_encodable() {
    assert_program_eq!(
        "fn main() -> f32 { 0.0 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::F_32, Address::new(Memory::ENCODED, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::F_32],
                register_count: 1,
                argument_count: 0,
            },
        ],
        return_type: DustType::F32
    );
}

#[test]
fn f32_not_encodable() {
    assert_program_eq!(
        "fn main() -> f32 { 3.40282347e+38 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::F_32, Address::new(Memory::CONSTANT, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::F_32],
                register_count: 1,
                argument_count: 0,
            },
        ],
        return_type: DustType::F32
    );
}

#[test]
fn f64_encodable() {
    assert_program_eq!(
        "fn main() -> f64 { 0.0 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::F_64, Address::new(Memory::ENCODED, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::F_64],
                register_count: 2,
                argument_count: 0,
            },
        ],
        return_type: DustType::F64
    );
}

#[test]
fn f64_not_encodable() {
    assert_program_eq!(
        "fn main() -> f64 { 1.7976931348623157e+308_f64 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::F_64, Address::new(Memory::CONSTANT, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::F_64],
                register_count: 2,
                argument_count: 0,
            },
        ],
        return_type: DustType::F64
    );
}

#[test]
fn character_encodable() {
    assert_program_eq!(
        "fn main() -> char { 'q' }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::CHARACTER, Address::new(Memory::ENCODED, 'q' as u16)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::CHARACTER],
                register_count: 1,
                argument_count: 0,
            },
        ],
        return_type: DustType::Character
    );
}

#[test]
fn character_not_encodable() {
    assert_program_eq!(
        "fn main() -> char { '🦀' }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::CHARACTER, Address::new(Memory::CONSTANT, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::CHARACTER],
                register_count: 1,
                argument_count: 0,
            },
        ],
        return_type: DustType::Character
    );
}
