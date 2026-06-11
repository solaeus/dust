use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::{DustStructTypeFields, DustType},
    instruction::{Address, Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn const_reference() {
    assert_program_eq!(
        "
            const VALUE: i64 = 42;
            fn main() -> i64 { VALUE }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 42)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 2,
                argument_count: 0,
            },
        ],
        return_type: DustType::I64
    );
}

#[test]
fn enum_variant_unit() {
    assert_program_eq!(
        "
            enum Thing { One, Two }
            fn main() -> Thing { Thing::One }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::U_16, Address::new(MemoryKind::ENCODED, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::U_16],
                register_count: 1,
                argument_count: 0,
            },
        ],
        return_type: DustType::Enum(Box::new(crate::dust_type::DustEnumType {
            name: "Thing".into(),
            variants: vec![
                ("One".into(), DustStructTypeFields::Unit),
                ("Two".into(), DustStructTypeFields::Unit),
            ],
        }))
    );
}
