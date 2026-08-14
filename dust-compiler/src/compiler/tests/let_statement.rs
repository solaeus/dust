use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Address, Instruction, Memory, OperandType},
    prototype::Prototype,
};

#[test]
fn constant() {
    assert_program_eq!(
        "
            fn main() -> u8 {
                let x = 42;
                x
            }
        ",
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
fn shadowed() {
    assert_program_eq!(
        "
            fn main() -> u8 {
                let x = 42;
                let x = x;
                x
            }
        ",
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
fn mutable() {
    assert_program_eq!(
        "
            fn main() -> u8 {
                let mut x = 41;
                x += 1;
                x
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::U_8, Address::new(Memory::ENCODED, 41)),
                    Instruction::add(0, OperandType::U_8, Address::new(Memory::REGISTER, 0), Address::new(Memory::ENCODED, 1)),
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
