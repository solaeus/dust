use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Instruction, OperandType},
    prototype::Prototype,
};

#[test]
fn array_literal() {
    assert_program_eq!(
        "fn main() -> [i64; 3] { [1, 2, 3] }",
        prototypes: [
            Prototype {
                instructions: vec![],
                return_types: smallvec![OperandType::I_64, OperandType::I_64, OperandType::I_64],
                register_count: 0,
                argument_count: 0,
            },
        ],
        return_type: DustType::Array(Box::new(DustType::I64), 3)
    );
}

#[test]
fn array_repeat() {
    assert_program_eq!(
        "fn main() -> [i64; 3] { [0; 3] }",
        prototypes: [
            Prototype {
                instructions: vec![],
                return_types: smallvec![OperandType::I_64, OperandType::I_64, OperandType::I_64],
                register_count: 0,
                argument_count: 0,
            },
        ],
        return_type: DustType::Array(Box::new(DustType::I64), 3)
    );
}

#[test]
fn index_expression() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let values = [10, 20, 30];
                values[1]
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
fn index_expression_runtime() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut values = [10, 20, 30];
                values[1]
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
fn index_with_runtime_index() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let values = [10, 20, 30];
                let mut position = 1;
                values[position]
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
