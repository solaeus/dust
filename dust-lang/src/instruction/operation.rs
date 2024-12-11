//! Part of an [Instruction][crate::Instruction] that is encoded as a single byte.

use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

/// Part of an [Instruction][crate::Instruction] that is encoded as a single byte.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Operation(pub u8);

impl Operation {
    pub const MOVE: Operation = Operation(0);
    pub const CLOSE: Operation = Operation(1);
    pub const LOAD_BOOLEAN: Operation = Operation(2);
    pub const LOAD_CONSTANT: Operation = Operation(3);
    pub const LOAD_LIST: Operation = Operation(4);
    pub const LOAD_SELF: Operation = Operation(5);
    pub const GET_LOCAL: Operation = Operation(6);
    pub const SET_LOCAL: Operation = Operation(7);
    pub const ADD: Operation = Operation(8);
    pub const SUBTRACT: Operation = Operation(9);
    pub const MULTIPLY: Operation = Operation(10);
    pub const DIVIDE: Operation = Operation(11);
    pub const MODULO: Operation = Operation(12);
    pub const TEST: Operation = Operation(13);
    pub const TEST_SET: Operation = Operation(14);
    pub const EQUAL: Operation = Operation(15);
    pub const LESS: Operation = Operation(16);
    pub const LESS_EQUAL: Operation = Operation(17);
    pub const NEGATE: Operation = Operation(18);
    pub const NOT: Operation = Operation(19);
    pub const CALL: Operation = Operation(20);
    pub const CALL_NATIVE: Operation = Operation(21);
    pub const JUMP: Operation = Operation(22);
    pub const RETURN: Operation = Operation(23);
}

impl Operation {
    pub fn name(self) -> &'static str {
        match self {
            Self::MOVE => "MOVE",
            Self::CLOSE => "CLOSE",
            Self::LOAD_BOOLEAN => "LOAD_BOOLEAN",
            Self::LOAD_CONSTANT => "LOAD_CONSTANT",
            Self::LOAD_LIST => "LOAD_LIST",
            Self::LOAD_SELF => "LOAD_SELF",
            Self::GET_LOCAL => "GET_LOCAL",
            Self::SET_LOCAL => "SET_LOCAL",
            Self::ADD => "ADD",
            Self::SUBTRACT => "SUBTRACT",
            Self::MULTIPLY => "MULTIPLY",
            Self::DIVIDE => "DIVIDE",
            Self::MODULO => "MODULO",
            Self::TEST => "TEST",
            Self::TEST_SET => "TEST_SET",
            Self::EQUAL => "EQUAL",
            Self::LESS => "LESS",
            Self::LESS_EQUAL => "LESS_EQUAL",
            Self::NEGATE => "NEGATE",
            Self::NOT => "NOT",
            Self::CALL => "CALL",
            Self::CALL_NATIVE => "CALL_NATIVE",
            Self::JUMP => "JUMP",
            Self::RETURN => "RETURN",
            _ => Self::panic_from_unknown_code(self.0),
        }
    }

    pub fn panic_from_unknown_code(code: u8) -> ! {
        panic!("Unknown operation code: {code}");
    }
}

impl Debug for Operation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
