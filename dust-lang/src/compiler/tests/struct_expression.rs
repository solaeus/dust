use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::{DustStructType, DustStructTypeFields, DustType},
    instruction::{Address, Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn construct_unit_struct() {
    assert_program_eq!(
        "
            struct Thing;
            fn main() -> Thing { Thing }
        ",
        prototypes: [
            Prototype {
                instructions: vec![],
                return_types: smallvec![],
                register_count: 0,
                argument_count: 0,
            },
        ],
        return_type: DustType::Struct(Box::new(DustStructType {
            name: "Thing".into(),
            value_type: DustStructTypeFields::Unit,
        }))
    );
}

#[test]
fn construct_named_fields() {
    assert_program_eq!(
        "
            struct Point { horizontal: i64, vertical: i64 }
            fn main() -> Point { Point { horizontal: 10, vertical: 20 } }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_32, Address::new(MemoryKind::ENCODED, 10)),
                    Instruction::r#move(2, OperandType::I_32, Address::new(MemoryKind::ENCODED, 20)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64, OperandType::I_64],
                register_count: 4,
                argument_count: 0,
            },
        ],
        return_type: DustType::Struct(Box::new(DustStructType {
            name: "Point".into(),
            value_type: DustStructTypeFields::Named(vec![
                ("horizontal".into(), DustType::I64),
                ("vertical".into(), DustType::I64),
            ]),
        }))
    );
}

#[test]
fn field_access() {
    assert_program_eq!(
        "
            struct Point { horizontal: i64, vertical: i64 }
            fn main() -> i64 {
                let point = Point { horizontal: 10, vertical: 20 };
                point.horizontal
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_32, Address::new(MemoryKind::ENCODED, 10)),
                    Instruction::r#move(2, OperandType::I_32, Address::new(MemoryKind::ENCODED, 20)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 4,
                argument_count: 0,
            },
        ],
        return_type: DustType::I64
    );
}

#[test]
fn field_access_runtime() {
    assert_program_eq!(
        "
            struct Point { horizontal: i64, vertical: i64 }
            fn main() -> i64 {
                let mut point = Point { horizontal: 10, vertical: 20 };
                point.horizontal
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_32, Address::new(MemoryKind::ENCODED, 10)),
                    Instruction::r#move(2, OperandType::I_32, Address::new(MemoryKind::ENCODED, 20)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 4,
                argument_count: 0,
            },
        ],
        return_type: DustType::I64
    );
}
