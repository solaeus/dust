use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Address, Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn boolean() {
    assert_program_eq!(
        "fn main() -> bool { true }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::BOOLEAN, Address::new(MemoryKind::ENCODED, true as u16)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::BOOLEAN],
                register_count: 1,
                argument_count: 0,
            }
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
                    Instruction::r#move(0, OperandType::I_8, Address::new( MemoryKind::ENCODED, (-42_i16) as u16)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_8],
                register_count: 1,
                argument_count: 0,
            }
        ],
        return_type: DustType::I8
    );
}

#[test]
fn i16() {
    todo!()
}

#[test]
fn i32() {
    todo!()
}

#[test]
fn i64_encodable() {
    assert_program_eq!(
        "fn main() -> i64 { 42 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 42)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 1,
                argument_count: 0,
            }
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
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::CONSTANT, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 1,
                argument_count: 0,
            }
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
                    Instruction::r#move(0, OperandType::I_128, Address::new(MemoryKind::ENCODED, 42)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_128],
                register_count: 1,
                argument_count: 0,
            }
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
                    Instruction::r#move(0, OperandType::I_128, Address::new(MemoryKind::CONSTANT, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_128],
                register_count: 1,
                argument_count: 0,
            }
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
                    Instruction::r#move(0, OperandType::U_8, Address::new(MemoryKind::ENCODED, 42)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::U_8],
                register_count: 1,
                argument_count: 0,
            }
        ],
        return_type: DustType::U8
    );
}

#[test]
fn u16() {
    todo!()
}

#[test]
fn u32() {
    todo!()
}

#[test]
fn u64() {
    todo!()
}

#[test]
fn u128() {
    todo!()
}

#[test]
fn f32() {
    todo!()
}

#[test]
fn f64_encodable() {
    todo!()
}

#[test]
fn f64_not_encodable() {
    todo!()
}

#[test]
fn character_encodable() {
    todo!()
}

#[test]
fn character_not_encodable() {
    todo!()
}
