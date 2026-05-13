use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Instruction, OperandType},
    prototype::Prototype,
};

#[test]
fn addition() {
    assert_program_eq!(
        "fn main() -> i64 { 1 + 2 }",
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
fn subtraction() {
    assert_program_eq!(
        "fn main() -> i64 { 3 - 1 }",
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
fn multiplication() {
    assert_program_eq!(
        "fn main() -> i64 { 2 * 3 }",
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
fn division() {
    assert_program_eq!(
        "fn main() -> i64 { 6 / 2 }",
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
fn modulo() {
    assert_program_eq!(
        "fn main() -> i64 { 7 % 3 }",
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
fn exponent() {
    assert_program_eq!(
        "fn main() -> i64 { 2 ^ 3 }",
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
                instructions: vec![],
                return_types: smallvec![OperandType::F_64],
                register_count: 0,
                argument_count: 0,
            },
        ],
        return_type: DustType::F64
    );
}
