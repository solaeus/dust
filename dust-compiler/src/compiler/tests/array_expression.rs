use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Address, Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn array_literal() {
    assert_program_eq!(
        "fn main() -> [i64; 3] { [1, 2, 3] }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 1)),
                    Instruction::r#move(2, OperandType::I_64, Address::new(MemoryKind::ENCODED, 2)),
                    Instruction::r#move(4, OperandType::I_64, Address::new(MemoryKind::ENCODED, 3)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64, OperandType::I_64, OperandType::I_64],
                register_count: 6,
                argument_count: 0,
            },
        ],
        return_type: DustType::Array(Box::new(DustType::I64), 3)
    );
}

#[test]
fn array_repeat() {
    assert_program_eq!(
        "fn main() -> [i64; 3] { [0; 3] }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 0)),
                    Instruction::r#move(2, OperandType::I_64, Address::new(MemoryKind::REGISTER, 0)),
                    Instruction::r#move(4, OperandType::I_64, Address::new(MemoryKind::REGISTER, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64, OperandType::I_64, OperandType::I_64],
                register_count: 6,
                argument_count: 0,
            },
        ],
        return_type: DustType::Array(Box::new(DustType::I64), 3)
    );
}

#[test]
fn index_expression() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let values = [10, 20, 30];
                values[1]
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 10)),
                    Instruction::r#move(2, OperandType::I_64, Address::new(MemoryKind::ENCODED, 20)),
                    Instruction::r#move(4, OperandType::I_64, Address::new(MemoryKind::ENCODED, 30)),
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::REGISTER, 2)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 6,
                argument_count: 0,
            },
        ],
        return_type: DustType::I64
    );
}

#[test]
fn index_with_runtime_index() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let values = [10, 20, 30];
                let mut position = 1;
                values[position]
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 10)),
                    Instruction::r#move(2, OperandType::I_64, Address::new(MemoryKind::ENCODED, 20)),
                    Instruction::r#move(4, OperandType::I_64, Address::new(MemoryKind::ENCODED, 30)),
                    Instruction::r#move(6, OperandType::I_32, Address::new(MemoryKind::ENCODED, 1)),
                    Instruction::get(0, OperandType::I_64, 0, Address::new(MemoryKind::REGISTER, 6)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 7,
                argument_count: 0,
            },
        ],
        return_type: DustType::I64
    );
}
