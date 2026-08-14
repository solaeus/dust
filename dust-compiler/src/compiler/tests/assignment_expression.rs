use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Address, Instruction, Memory, OperandType},
    prototype::Prototype,
};

#[test]
fn simple_assignment() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut destination = 0;
                destination = 42;
                destination
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 0)),
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
fn assignment_from_runtime() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut destination = 0;
                let mut source = 42;
                destination = source;
                destination
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 0)),
                    Instruction::r#move(2, OperandType::I_64, Address::new(Memory::ENCODED, 42)),
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::REGISTER, 2)),
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
