use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Instruction, OperandType},
    prototype::Prototype,
};

#[test]
fn while_loop() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut count = 0;
                let mut iteration = 0;
                while iteration < 5 {
                    count += 1;
                    iteration += 1;
                }
                count
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
fn while_with_runtime_condition() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut count = 0;
                let mut limit = 3;
                while count < limit {
                    count += 1;
                }
                count
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
fn break_without_value() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut count = 0;
                while count < 10 {
                    if count == 3 { break; }
                    count += 1;
                }
                count
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
fn break_with_value() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut count = 0;
                while count < 10 {
                    if count == 5 { break count; }
                    count += 1;
                }
                count
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
