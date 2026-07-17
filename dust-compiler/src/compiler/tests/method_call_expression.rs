use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Address, Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn method_call() {
    assert_program_eq!(
        "
            struct Value { inner: i64 }

            impl Value {
                fn get(self) -> i64 { self.inner }
            }

            fn main() -> i64 {
                let instance = Value { inner: 42 };
                instance.get()
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 42)),
                    Instruction::call(0, Address::new(MemoryKind::CONSTANT, 1), 0),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 2,
                argument_count: 0,
            },
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::REGISTER, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 2,
                argument_count: 2,
            },
        ],
        return_type: DustType::I64
    );
}

#[test]
fn method_call_runtime() {
    assert_program_eq!(
        "
            struct Value { inner: i64 }

            impl Value {
                fn get(self) -> i64 { self.inner }
            }

            fn main() -> i64 {
                let mut instance = Value { inner: 42 };
                instance.get()
            }
        ",
        prototypes: [
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::ENCODED, 42)),
                    Instruction::call(0, Address::new(MemoryKind::CONSTANT, 1), 0),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 2,
                argument_count: 0,
            },
            Prototype {
                instructions: vec![
                    Instruction::r#move(0, OperandType::I_64, Address::new(MemoryKind::REGISTER, 0)),
                    Instruction::r#return(),
                ],
                return_types: smallvec![OperandType::I_64],
                register_count: 2,
                argument_count: 2,
            },
        ],
        return_type: DustType::I64
    );
}
