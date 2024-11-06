//! Part of an [Instruction][crate::Instruction], which can be executed by the Dust virtual machine.
//!
//! !!! Warning !!!
//! The byte values of the operations matter. The seventh and eighth bits must be zero so that the
//! [Instruction][crate::Instruction] type can use them as flags.

use std::fmt::{self, Display, Formatter};

macro_rules! define_operation {
    ($(($name:ident, $byte:literal, $str:expr, $type:expr)),*) => {
        /// Part of an [Instruction][crate::Instruction], which can be executed by the Dust virtual machine.)
        ///
        /// See the [module-level documentation](index.html) for more information.
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum Operation {
            $(
                $name = $byte as isize,
            )*
        }

        impl From<u8> for Operation {
            fn from(byte: u8) -> Self {
                match byte {
                    $(
                        $byte => Operation::$name,
                    )*
                    _ => {
                        if cfg!(test) {
                            panic!("Invalid operation byte: {}", byte)
                        } else {
                            Operation::Return
                        }
                    }
                }
            }
        }

        impl From<Operation> for u8 {
            fn from(operation: Operation) -> Self {
                match operation {
                    $(
                        Operation::$name => $byte,
                    )*
                }
            }
        }

        impl Display for Operation {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                match self {
                    $(
                        Operation::$name => write!(f, "{}", $str),
                    )*
                }
            }
        }
    }
}

define_operation! {
    (Move, 0b0000_0000, "MOVE", None),
    (Close, 0b000_0001, "CLOSE", None),
    (LoadBoolean, 0b0000_0010, "LOAD_BOOLEAN", None),
    (LoadConstant, 0b0000_0011, "LOAD_CONSTANT", None),
    (LoadList, 0b0000_0100, "LOAD_LIST", None),
    (LoadSelf, 0b0000_0101, "LOAD_SELF", None),
    (DefineLocal, 0b0000_0110, "DEFINE_LOCAL", None),
    (GetLocal, 0b0000_0111, "GET_LOCAL", None),
    (SetLocal, 0b0000_1000, "SET_LOCAL", None),
    (Add, 0b0000_1001, "ADD", None),
    (Subtract, 0b0000_1010, "SUBTRACT", None),
    (Multiply, 0b0000_1011, "MULTIPLY", None),
    (Divide, 0b0000_1100, "DIVIDE", None),
    (Modulo, 0b0000_1101, "MODULO", None),
    (Test, 0b0000_1110, "TEST", None),
    (TestSet, 0b0000_1111, "TEST_SET", None),
    (Equal, 0b0001_0000, "EQUAL", None),
    (Less, 0b0001_0001, "LESS", None),
    (LessEqual, 0b0001_0010, "LESS_EQUAL", None),
    (Negate, 0b0001_0011, "NEGATE", None),
    (Not, 0b0001_0100, "NOT", None),
    (Jump, 0b0001_0101, "JUMP", None),
    (Call, 0b0001_0110, "CALL", None),
    (CallNative, 0b0001_0111, "CALL_NATIVE", None),
    (Return, 0b0001_1000, "RETURN", None)
}

impl Operation {
    pub fn is_math(&self) -> bool {
        matches!(
            self,
            Operation::Add
                | Operation::Subtract
                | Operation::Multiply
                | Operation::Divide
                | Operation::Modulo
        )
    }

    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            Operation::Equal | Operation::Less | Operation::LessEqual
        )
    }

    pub fn is_test(&self) -> bool {
        matches!(self, Operation::Test | Operation::TestSet)
    }
}
