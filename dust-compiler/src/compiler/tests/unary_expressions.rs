use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Address, Instruction, Memory, OperandType},
    prototype::Prototype,
};

#[test]
fn negation_integer() {
    assert_program_eq!(
        "fn main() -> i64 { -42 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 65494)),
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
fn negation_float() {
    assert_program_eq!(
        "fn main() -> f64 { -3.14 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::F_64, Address::new(Memory::CONSTANT, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::F_64],
                register_count: 2,
                argument_count: 0,
            },
        ],
        return_type: DustType::F64
    );
}

#[test]
fn not_true() {
    assert_program_eq!(
        "fn main() -> bool { !true }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::BOOLEAN, Address::new(Memory::ENCODED, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::BOOLEAN],
                register_count: 1,
                argument_count: 0,
            },
        ],
        return_type: DustType::Boolean
    );
}

#[test]
fn not_false() {
    assert_program_eq!(
        "fn main() -> bool { !false }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::BOOLEAN, Address::new(Memory::ENCODED, 1)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::BOOLEAN],
                register_count: 1,
                argument_count: 0,
            },
        ],
        return_type: DustType::Boolean
    );
}

#[test]
fn runtime_negation() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut value = 42;
                -value
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 42)),
                    Instruction::negate(0, OperandType::I_64, Address::new(Memory::REGISTER, 0)),
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
fn runtime_not() {
    assert_program_eq!(
        "
            fn main() -> bool {
                let mut flag = true;
                !flag
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::BOOLEAN, Address::new(Memory::ENCODED, 1)),
                    Instruction::negate(0, OperandType::BOOLEAN, Address::new(Memory::REGISTER, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::BOOLEAN],
                register_count: 1,
                argument_count: 0,
            },
        ],
        return_type: DustType::Boolean
    );
}
