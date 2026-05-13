use smallvec::smallvec;

use crate::{
    assert_program_eq,
    dust_type::DustType,
    instruction::{Instruction, OperandType},
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
                instructions: vec![],
                return_types: smallvec![OperandType::I_64],
                register_count: 0,
                argument_count: 0,
            },
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
                instructions: vec![],
                return_types: smallvec![OperandType::I_64],
                register_count: 0,
                argument_count: 0,
            },
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
