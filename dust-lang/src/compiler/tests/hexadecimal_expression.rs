use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Instruction, OperandType},
    prototype::Prototype,
};

#[test]
fn hexadecimal_literal() {
    assert_program_eq!(
        "fn main() -> u8 { 0x2A }",
        prototypes: [
            Prototype {
                instructions: vec![],
                return_types: smallvec![OperandType::U_8],
                register_count: 0,
                argument_count: 0,
            },
        ],
        return_type: DustType::U8
    );
}
