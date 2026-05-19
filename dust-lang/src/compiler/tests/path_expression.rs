use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
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
            enum Option { None, Some }
            fn main() -> Option { None }
        ",
        prototypes: [
            Prototype {
                instructions: vec![],
                return_types: smallvec![OperandType::I_64],
                register_count: 0,
                argument_count: 0,
            },
        ],
        return_type: DustType::Enum(Box::new(crate::dust_type::DustEnumType {
            name: "Option".into(),
            variants: vec![
                ("None".into(), crate::dust_type::DustStructTypeFields::Unit),
                ("Some".into(), crate::dust_type::DustStructTypeFields::Unit),
            ],
        }))
    );
}
