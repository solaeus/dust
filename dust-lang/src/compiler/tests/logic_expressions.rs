use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Instruction, OperandType},
    prototype::Prototype,
};

#[test]
fn and() {
    assert_program_eq!(
        "fn main() -> bool { true && false }",
        prototypes: [
            Prototype {
                instructions: vec![],
                return_types: smallvec![OperandType::BOOLEAN],
                register_count: 0,
                argument_count: 0,
            },
        ],
        return_type: DustType::Boolean
    );
}

#[test]
fn or() {
    assert_program_eq!(
        "fn main() -> bool { true || false }",
        prototypes: [
            Prototype {
                instructions: vec![],
                return_types: smallvec![OperandType::BOOLEAN],
                register_count: 0,
                argument_count: 0,
            },
        ],
        return_type: DustType::Boolean
    );
}

#[test]
fn runtime_and() {
    assert_program_eq!(
        "
            fn main() -> bool {
                let mut left = true;
                let mut right = false;
                left && right
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![],
                return_types: smallvec![OperandType::BOOLEAN],
                register_count: 0,
                argument_count: 0,
            },
        ],
        return_type: DustType::Boolean
    );
}

#[test]
fn runtime_or() {
    assert_program_eq!(
        "
            fn main() -> bool {
                let mut left = false;
                let mut right = true;
                left || right
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![],
                return_types: smallvec![OperandType::BOOLEAN],
                register_count: 0,
                argument_count: 0,
            },
        ],
        return_type: DustType::Boolean
    );
}
