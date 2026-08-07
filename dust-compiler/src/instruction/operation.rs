use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Operation(pub u8);

impl Operation {
    pub const NO_OP: Operation = Operation(0);

    // Memory operations
    pub const MOVE: Operation = Operation(1);
    pub const REFERENCE: Operation = Operation(2);
    pub const GET: Operation = Operation(3);
    pub const SET: Operation = Operation(4);
    pub const DROP: Operation = Operation(5);

    // Comparison binary operations (with control flow)
    pub const EQUAL: Operation = Operation(6);
    pub const NOT_EQUAL: Operation = Operation(7);
    pub const GREATER: Operation = Operation(8);
    pub const GREATER_EQUAL: Operation = Operation(9);
    pub const LESS: Operation = Operation(10);
    pub const LESS_EQUAL: Operation = Operation(11);

    // Logical AND/OR (with control flow)
    pub const TEST: Operation = Operation(12);

    // Logical NOT and negation
    pub const NEGATE: Operation = Operation(13);

    // Arithmetic operations
    pub const ADD: Operation = Operation(14);
    pub const SUBTRACT: Operation = Operation(15);
    pub const MULTIPLY: Operation = Operation(16);
    pub const DIVIDE: Operation = Operation(17);
    pub const MODULO: Operation = Operation(18);
    pub const EXPONENT: Operation = Operation(19);

    // Function calls
    pub const CALL: Operation = Operation(20);
    pub const CALL_NATIVE: Operation = Operation(21);

    // Control flow
    pub const JUMP: Operation = Operation(22);
    pub const RETURN: Operation = Operation(23);

    // Type conversion
    pub const CONVERT: Operation = Operation(24);
}

impl Operation {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::NO_OP => "NO_OP",
            Self::MOVE => "MOVE",
            Self::REFERENCE => "REFERENCE",
            Self::GET => "GET",
            Self::SET => "SET",
            Self::DROP => "DROP",
            Self::ADD => "ADD",
            Self::SUBTRACT => "SUBTRACT",
            Self::MULTIPLY => "MULTIPLY",
            Self::DIVIDE => "DIVIDE",
            Self::MODULO => "MODULO",
            Self::EXPONENT => "POWER",
            Self::EQUAL => "EQUAL",
            Self::LESS => "LESS",
            Self::LESS_EQUAL => "LESS_EQUAL",
            Self::NEGATE => "NEGATE",
            Self::TEST => "TEST",
            Self::CALL => "CALL",
            Self::CALL_NATIVE => "CALL_NATIVE",
            Self::JUMP => "JUMP",
            Self::RETURN => "RETURN",
            Self::CONVERT => "CONVERT",
            _ => "UNKNOWN",
        }
    }
}

impl Debug for Operation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
