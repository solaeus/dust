use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Address, Instruction, Memory, OperandType},
    prototype::Prototype,
};

#[test]
fn empty_block() {
    assert_program_eq!(
        "fn main() { }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#return(),
                ],
                return_types: smallvec![],
                register_count: 0,
                argument_count: 0,
            },
        ],
        return_type: DustType::Unit
    );
}

#[test]
fn nested_block() {
    assert_program_eq!(
        "fn main() -> i64 { { 42 } }",
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
fn block_with_statement() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                {
                    let base = 10;
                    base + 32
                }
            }
        ",
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
