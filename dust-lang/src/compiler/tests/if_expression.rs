use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Address, Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn if_branch() {
    assert_program_eq!(
        "fn main() -> i64 { if true { 1 } else { 0 } }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::test(true, Address::new(MemoryKind::ENCODED, 1), 2),
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 1)),
                    Instruction::jump(1, true),
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 0)),
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
fn if_else_branch() {
    assert_program_eq!(
        "fn main() -> bool { if 1 < 2 { true } else { false } }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::test(true, Address::new(MemoryKind::ENCODED, 1), 2),
                    Instruction::r#move(0, OperandType::BOOLEAN, Address::new(MemoryKind::ENCODED, 1)),
                    Instruction::jump(1, true),
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
fn if_else_if_else() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut value = 2;
                if value == 1 { 10 } else if value == 2 { 20 } else { 30 }
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_32, Address::new(MemoryKind::ENCODED, 2)),
                    Instruction::equal(true, OperandType::I_32, Address::new(MemoryKind::REGISTER, 0), Address::new(MemoryKind::ENCODED, 1)),
                    Instruction::jump(2, true),
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 10)),
                    Instruction::jump(5, true),
                    Instruction::equal(true, OperandType::I_32, Address::new(MemoryKind::REGISTER, 0), Address::new(MemoryKind::ENCODED, 2)),
                    Instruction::jump(2, true),
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 20)),
                    Instruction::jump(1, true),
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 30)),
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
fn if_with_runtime_condition() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut condition = true;
                if condition { 1 } else { 0 }
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::BOOLEAN, Address::new(MemoryKind::ENCODED, 1)),
                    Instruction::test(true, Address::new(MemoryKind::REGISTER, 0), 2),
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 1)),
                    Instruction::jump(1, true),
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 0)),
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
