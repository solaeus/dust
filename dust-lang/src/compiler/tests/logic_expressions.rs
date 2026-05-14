use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Address, Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn and() {
    assert_program_eq!(
        "fn main() -> bool { true && false }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::BOOLEAN, Address::new(MemoryKind::ENCODED, 0)),
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
fn or() {
    assert_program_eq!(
        "fn main() -> bool { true || false }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::BOOLEAN, Address::new(MemoryKind::ENCODED, 1)),
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
fn runtime_and() {
    assert_program_eq!(
        "
            fn main() -> bool {
                let mut left = true;
                let mut right = false;
                left && right
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::BOOLEAN, Address::new(MemoryKind::ENCODED, 1)),
                    Instruction::r#move(1, OperandType::BOOLEAN, Address::new(MemoryKind::ENCODED, 0)),
                    Instruction::test(false, Address::new(MemoryKind::REGISTER, 0), 1),
                    Instruction::move_with_jump(0, OperandType::BOOLEAN, Address::new(MemoryKind::REGISTER, 1), 1, true),
                    Instruction::r#move(0, OperandType::BOOLEAN, Address::new(MemoryKind::REGISTER, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::BOOLEAN],
                register_count: 2,
                argument_count: 0,
            },
        ],
        return_type: DustType::Boolean
    );
}

#[test]
fn runtime_or() {
    assert_program_eq!(
        "
            fn main() -> bool {
                let mut left = false;
                let mut right = true;
                left || right
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::BOOLEAN, Address::new(MemoryKind::ENCODED, 0)),
                    Instruction::r#move(1, OperandType::BOOLEAN, Address::new(MemoryKind::ENCODED, 1)),
                    Instruction::test(true, Address::new(MemoryKind::REGISTER, 0), 1),
                    Instruction::move_with_jump(0, OperandType::BOOLEAN, Address::new(MemoryKind::REGISTER, 1), 1, true),
                    Instruction::r#move(0, OperandType::BOOLEAN, Address::new(MemoryKind::REGISTER, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::BOOLEAN],
                register_count: 2,
                argument_count: 0,
            },
        ],
        return_type: DustType::Boolean
    );
}
