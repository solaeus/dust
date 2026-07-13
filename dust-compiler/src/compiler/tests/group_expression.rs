use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Address, Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn grouped_expression() {
    assert_program_eq!(
        "fn main() -> i64 { (42) }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 42)),
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
fn grouped_with_operator() {
    assert_program_eq!(
        "fn main() -> i64 { (1 + 2) * 3 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 9)),
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
