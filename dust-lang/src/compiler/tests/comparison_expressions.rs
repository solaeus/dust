use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Instruction, OperandType},
    prototype::Prototype,
};

#[test]
fn equal() {
    assert_program_eq!(
        "fn main() -> bool { 1 == 1 }",
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
fn not_equal() {
    assert_program_eq!(
        "fn main() -> bool { 1 != 2 }",
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
fn less_than() {
    assert_program_eq!(
        "fn main() -> bool { 1 < 2 }",
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
fn less_than_or_equal() {
    assert_program_eq!(
        "fn main() -> bool { 1 <= 1 }",
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
fn greater_than() {
    assert_program_eq!(
        "fn main() -> bool { 2 > 1 }",
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
fn greater_than_or_equal() {
    assert_program_eq!(
        "fn main() -> bool { 2 >= 2 }",
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
fn float_comparison() {
    assert_program_eq!(
        "fn main() -> bool { 3.0 > 1.5 }",
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
fn runtime_less_than() {
    assert_program_eq!(
        "
            fn main() -> bool {
                let mut left = 1;
                let mut right = 2;
                left < right
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
fn runtime_equal() {
    assert_program_eq!(
        "
            fn main() -> bool {
                let mut left = 42;
                let mut right = 42;
                left == right
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
