use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Address, Instruction, Memory, OperandType},
    prototype::Prototype,
};

#[test]
fn addition() {
    assert_program_eq!(
        "fn main() -> i64 { 1 + 2 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address { memory: Memory::ENCODED, index: 3 }),
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
fn subtraction() {
    assert_program_eq!(
        "fn main() -> i64 { 3 - 1 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 2)),
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
fn multiplication() {
    assert_program_eq!(
        "fn main() -> i64 { 2 * 3 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 6)),
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
fn division() {
    assert_program_eq!(
        "fn main() -> i64 { 6 / 2 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 3)),
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
fn modulo() {
    assert_program_eq!(
        "fn main() -> i64 { 7 % 3 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 1)),
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
fn exponent() {
    assert_program_eq!(
        "fn main() -> i64 { 2 ^ 3 }",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 8)),
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
fn addition_assignment() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut value = 1;
                value += 2;
                value
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 1)),
                    Instruction::add(0, OperandType::I_64, Address::new(Memory::REGISTER, 0), Address::new(Memory::ENCODED, 2)),
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
fn subtraction_assignment() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut value = 3;
                value -= 1;
                value
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 3)),
                    Instruction::subtract(0, OperandType::I_64, Address::new(Memory::REGISTER, 0), Address::new(Memory::ENCODED, 1)),
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
fn multiplication_assignment() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut value = 2;
                value *= 3;
                value
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 2)),
                    Instruction::multiply(0, OperandType::I_64, Address::new(Memory::REGISTER, 0), Address::new(Memory::ENCODED, 3)),
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
fn division_assignment() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut value = 6;
                value /= 2;
                value
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 6)),
                    Instruction::divide(0, OperandType::I_64, Address::new(Memory::REGISTER, 0), Address::new(Memory::ENCODED, 2)),
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
fn modulo_assignment() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut value = 7;
                value %= 3;
                value
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 7)),
                    Instruction::modulo(0, OperandType::I_64, Address::new(Memory::REGISTER, 0), Address::new(Memory::ENCODED, 3)),
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
fn exponent_assignment() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut value = 2;
                value ^= 3;
                value
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 2)),
                    Instruction::power(0, OperandType::I_64, Address::new(Memory::REGISTER, 0), Address::new(Memory::ENCODED, 3)),
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
fn runtime_addition() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut left = 1;
                let mut right = 2;
                left + right
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 1)),
                    Instruction::r#move(2, OperandType::I_64, Address::new(Memory::ENCODED, 2)),
                    Instruction::add(0, OperandType::I_64, Address::new(Memory::REGISTER, 0), Address::new(Memory::REGISTER, 2)),
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
fn runtime_multiplication() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut left = 3;
                let mut right = 4;
                left * right
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 3)),
                    Instruction::r#move(2, OperandType::I_64, Address::new(Memory::ENCODED, 4)),
                    Instruction::multiply(0, OperandType::I_64, Address::new(Memory::REGISTER, 0), Address::new(Memory::REGISTER, 2)),
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
fn runtime_addition_assignment() {
    assert_program_eq!(
        "
            fn main() -> i64 {
                let mut value = 1;
                let mut increment = 2;
                value += increment;
                value
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(Memory::ENCODED, 1)),
                    Instruction::r#move(2, OperandType::I_64, Address::new(Memory::ENCODED, 2)),
                    Instruction::add(0, OperandType::I_64, Address::new(Memory::REGISTER, 0), Address::new(Memory::REGISTER, 2)),
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
fn runtime_float_math() {
    assert_program_eq!(
        "
            fn main() -> f64 {
                let mut left = 2.5;
                let mut right = 3.5;
                left + right
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::F_64, Address::new(Memory::CONSTANT, 0)),
                    Instruction::r#move(2, OperandType::F_64, Address::new(Memory::CONSTANT, 2)),
                    Instruction::add(0, OperandType::F_64, Address::new(Memory::REGISTER, 0), Address::new(Memory::REGISTER, 2)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::F_64],
                register_count: 4,
                argument_count: 0,
            },
        ],
        return_type: DustType::F64
    );
}
