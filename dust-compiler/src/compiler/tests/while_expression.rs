use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Address, Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn while_loop() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut count = 0;
                let mut iteration = 0;
                while iteration < 5 {
                    count += 1;
                    iteration += 1;
                }
                count
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 0)),
                    Instruction::r#move(2, OperandType::I_32, Address::new(MemoryKind::ENCODED, 0)),
                    Instruction::less(true, OperandType::I_32, Address::new(MemoryKind::REGISTER, 2), Address::new(MemoryKind::ENCODED, 5)),
                    Instruction::jump(3, true),
                    Instruction::add(0, OperandType::I_64, Address::new(MemoryKind::REGISTER, 0), Address::new(MemoryKind::ENCODED, 1)),
                    Instruction::add(2, OperandType::I_32, Address::new(MemoryKind::REGISTER, 2), Address::new(MemoryKind::ENCODED, 1)),
                    Instruction::jump(3, false),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 3,
                argument_count: 0,
            },
        ],
        return_type: DustType::I64
    );
}

#[test]
fn while_with_runtime_condition() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut count = 0;
                let mut limit = 3;
                while count < limit {
                    count += 1;
                }
                count
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 0)),
                    Instruction::r#move(2, OperandType::I_64, Address::new(MemoryKind::ENCODED, 3)),
                    Instruction::less(true, OperandType::I_64, Address::new(MemoryKind::REGISTER, 0), Address::new(MemoryKind::REGISTER, 2)),
                    Instruction::jump(2, true),
                    Instruction::add(0, OperandType::I_64, Address::new(MemoryKind::REGISTER, 0), Address::new(MemoryKind::ENCODED, 1)),
                    Instruction::jump(2, false),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 4,
                argument_count: 0,
            },
        ],
        return_type: DustType::I64
    );
}

#[test]
fn break_without_value() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut count = 0;
                while count < 10 {
                    if count == 3 { break; }
                    count += 1;
                }
                count
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 0)),
                    Instruction::less(true, OperandType::I_64, Address::new(MemoryKind::REGISTER, 0), Address::new(MemoryKind::ENCODED, 10)),
                    Instruction::jump(5, true),
                    Instruction::equal(true, OperandType::I_64, Address::new(MemoryKind::REGISTER, 0), Address::new(MemoryKind::ENCODED, 3)),
                    Instruction::jump(1, true),
                    Instruction::jump(2, true),
                    Instruction::add(0, OperandType::I_64, Address::new(MemoryKind::REGISTER, 0), Address::new(MemoryKind::ENCODED, 1)),
                    Instruction::jump(5, false),
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
fn break_with_value() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut count = 0;
                while count < 10 {
                    if count == 5 { break; }
                    count += 1;
                }
                count
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 0)),
                    Instruction::less(true, OperandType::I_64, Address::new(MemoryKind::REGISTER, 0), Address::new(MemoryKind::ENCODED, 10)),
                    Instruction::jump(5, true),
                    Instruction::equal(true, OperandType::I_64, Address::new(MemoryKind::REGISTER, 0), Address::new(MemoryKind::ENCODED, 5)),
                    Instruction::jump(1, true),
                    Instruction::jump(2, true),
                    Instruction::add(0, OperandType::I_64, Address::new(MemoryKind::REGISTER, 0), Address::new(MemoryKind::ENCODED, 1)),
                    Instruction::jump(5, false),
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
