use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::{DustStructType, DustStructTypeFields, DustType},
    instruction::{Instruction, OperandType},
    prototype::Prototype,
};

#[test]
fn exclusive_range() {
    assert_program_eq!(
        "fn main() { 0..10 }",
        prototypes: [
            Prototype {
                instructions: vec![],
                return_types: smallvec![OperandType::I_64, OperandType::I_64],
                register_count: 0,
                argument_count: 0,
            },
        ],
        return_type: DustType::Struct(Box::new(DustStructType {
            name: "Range".into(),
            value_type: DustStructTypeFields::Named(vec![
                ("start".into(), DustType::I64),
                ("end".into(), DustType::I64),
            ]),
        }))
    );
}

#[test]
fn inclusive_range() {
    assert_program_eq!(
        "fn main() { 0..=10 }",
        prototypes: [
            Prototype {
                instructions: vec![],
                return_types: smallvec![OperandType::I_64, OperandType::I_64],
                register_count: 0,
                argument_count: 0,
            },
        ],
        return_type: DustType::Struct(Box::new(DustStructType {
            name: "RangeInclusive".into(),
            value_type: DustStructTypeFields::Named(vec![
                ("start".into(), DustType::I64),
                ("end".into(), DustType::I64),
            ]),
        }))
    );
}

#[test]
fn runtime_range() {
    assert_program_eq!(
        "
            fn main() {
                let mut start = 1;
                let mut end = 5;
                start..end
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![],
                return_types: smallvec![OperandType::I_64, OperandType::I_64],
                register_count: 0,
                argument_count: 0,
            },
        ],
        return_type: DustType::Struct(Box::new(DustStructType {
            name: "Range".into(),
            value_type: DustStructTypeFields::Named(vec![
                ("start".into(), DustType::I64),
                ("end".into(), DustType::I64),
            ]),
        }))
    );
}
