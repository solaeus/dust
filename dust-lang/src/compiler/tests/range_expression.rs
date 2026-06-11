use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::{DustStructType, DustStructTypeFields, DustType},
    instruction::{Address, Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn exclusive_range() {
    assert_program_eq!(
        "fn main() -> Range<i64> { 0..10 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 0)),
                    Instruction::r#move(2, OperandType::I_64, Address::new(MemoryKind::ENCODED, 10)),
                    Instruction::r#return(),
                ],
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
        "fn main() -> RangeInclusive<i64> { 0..=10 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 0)),
                    Instruction::r#move(2, OperandType::I_64, Address::new(MemoryKind::ENCODED, 10)),
                    Instruction::r#return(),
                ],
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
            fn main() -> Range<i64> {
                let mut start = 1;
                let mut end = 5;
                start..end
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 1)),
                    Instruction::r#move(2, OperandType::I_64, Address::new(MemoryKind::ENCODED, 5)),
                    Instruction::r#return(),
                ],
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
