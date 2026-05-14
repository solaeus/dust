use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Address, Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn call_no_arguments() {
    assert_program_eq!(
        "
            fn one() -> i64 { 1 }
            fn main() -> i64 { one() }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::call(0, Address::new(MemoryKind::ENCODED, 1), u16::MAX),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 2,
                argument_count: 0,
            },
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 1)),
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
fn call_one_argument() {
    assert_program_eq!(
        "
            fn double(value: i64) -> i64 { value * 2 }
            fn main() -> i64 { double(21) }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 21)),
                    Instruction::call(0, Address::new(MemoryKind::ENCODED, 1), 0),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 2,
                argument_count: 0,
            },
            Prototype {
                instructions: vec![
                    Instruction::multiply(0, OperandType::I_64, Address::new(MemoryKind::REGISTER, 0), Address::new(MemoryKind::ENCODED, 2)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 2,
                argument_count: 2,
            },
        ],
        return_type: DustType::I64
    );
}

#[test]
fn call_two_arguments() {
    assert_program_eq!(
        "
            fn add(left: i64, right: i64) -> i64 { left + right }
            fn main() -> i64 { add(10, 20) }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 10)),
                    Instruction::r#move(2, OperandType::I_64, Address::new(MemoryKind::ENCODED, 20)),
                    Instruction::call(0, Address::new(MemoryKind::ENCODED, 1), 0),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 4,
                argument_count: 0,
            },
            Prototype {
                instructions: vec![
                    Instruction::add(0, OperandType::I_64, Address::new(MemoryKind::REGISTER, 0), Address::new(MemoryKind::REGISTER, 2)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 4,
                argument_count: 4,
            },
        ],
        return_type: DustType::I64
    );
}

#[test]
fn call_with_runtime_arguments() {
    assert_program_eq!(
        "
            fn add(left: i64, right: i64) -> i64 { left + right }
            fn main() -> i64 {
                let mut first = 10;
                let mut second = 20;
                add(first, second)
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 10)),
                    Instruction::r#move(2, OperandType::I_64, Address::new(MemoryKind::ENCODED, 20)),
                    Instruction::call(0, Address::new(MemoryKind::ENCODED, 1), 0),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 4,
                argument_count: 0,
            },
            Prototype {
                instructions: vec![
                    Instruction::add(0, OperandType::I_64, Address::new(MemoryKind::REGISTER, 0), Address::new(MemoryKind::REGISTER, 2)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 4,
                argument_count: 4,
            },
        ],
        return_type: DustType::I64
    );
}
