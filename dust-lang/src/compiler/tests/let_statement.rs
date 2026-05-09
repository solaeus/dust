use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Address, Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn constant() {
    assert_program_eq!(
        "fn main() -> u8 { let x = 42; x }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::U_8, Address::new(MemoryKind::ENCODED, 42)),
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
fn runtime() {
    assert_program_eq!(
        r"
            fn main() -> u8 { foo(42) }
            fn foo(x: u8) -> u8 { let y = x; y }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::U_8, Address::new(MemoryKind::ENCODED, 42)),
                    Instruction::call(0, Address::new(MemoryKind::ENCODED, 1), 0),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::U_8],
                register_count: 1,
                argument_count: 0,
            },
            Prototype {
                instructions: vec![
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
