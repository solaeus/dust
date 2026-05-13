use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Instruction, OperandType},
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
                instructions: vec![],
                return_types: smallvec![OperandType::I_64],
                register_count: 0,
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
                instructions: vec![],
                return_types: smallvec![OperandType::I_64],
                register_count: 0,
                argument_count: 0,
            },
        ],
        return_type: DustType::I64
    );
}
