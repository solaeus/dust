//! Part of an [Instruction][crate::Instruction] that is encoded as a single byte.

use std::fmt::{self, Display, Formatter};

macro_rules! define_operation {
    ($(($name:ident, $byte:literal, $str:expr)),*) => {
        /// Part of an [Instruction][crate::Instruction] that is encoded as a single byte.
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
    (Move, 0, "MOVE"),
    (Close, 1, "CLOSE"),

    (LoadBoolean, 2, "LOAD_BOOLEAN"),
    (LoadConstant, 3, "LOAD_CONSTANT"),
    (LoadList, 4, "LOAD_LIST"),
    (LoadSelf, 5, "LOAD_SELF"),

    (DefineLocal, 6, "DEFINE_LOCAL"),
    (GetLocal, 7, "GET_LOCAL"),
    (SetLocal, 8, "SET_LOCAL"),

    (Add, 9, "ADD"),
    (Subtract, 10, "SUBTRACT"),
    (Multiply, 11, "MULTIPLY"),
    (Divide, 12, "DIVIDE"),
    (Modulo, 13, "MODULO"),

    (Test, 14, "TEST"),
    (TestSet, 15, "TEST_SET"),

    (Equal, 16, "EQUAL"),
    (Less, 17, "LESS"),
    (LessEqual, 18, "LESS_EQUAL"),

    (Negate, 19, "NEGATE"),
    (Not, 20, "NOT"),

    (Call, 21, "CALL"),
    (CallNative, 22, "CALL_NATIVE"),

    (Jump, 23, "JUMP"),
    (Return, 24, "RETURN")
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
