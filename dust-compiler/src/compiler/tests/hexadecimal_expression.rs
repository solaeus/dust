use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Address, Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn hexadecimal_literal() {
    assert_program_eq!(
        "fn main() -> u8 { 0x2A }",
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
