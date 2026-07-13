use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Operation(pub u8);

impl Operation {
    pub const NO_OP: Operation = Operation(0);

    // Memory manipulation (with optional control flow)
    pub const MOVE: Operation = Operation(1);
    pub const DROP: Operation = Operation(2);

    // Arithmetic binary operations
    pub const ADD: Operation = Operation(6);
    pub const SUBTRACT: Operation = Operation(7);
    pub const MULTIPLY: Operation = Operation(8);
    pub const DIVIDE: Operation = Operation(9);
    pub const MODULO: Operation = Operation(10);
    pub const POWER: Operation = Operation(11);

    // Comparison binary operations (with control flow)
    pub const EQUAL: Operation = Operation(12);
    pub const LESS: Operation = Operation(13);
    pub const LESS_EQUAL: Operation = Operation(14);

    // Logical AND/OR operations (with control flow)
    pub const TEST: Operation = Operation(15);

    // Unary numeric negation and logical NOT
    pub const NEGATE: Operation = Operation(16);

    // Function calls
    pub const CALL: Operation = Operation(17);
    pub const CALL_NATIVE: Operation = Operation(18);

    // Control flow
    pub const JUMP: Operation = Operation(19);
    pub const RETURN: Operation = Operation(20);

    // Type conversions
    pub const CONVERT: Operation = Operation(21);

    // Indexed access
    pub const CHECK_INDEX: Operation = Operation(22);
    pub const GET_INDEX: Operation = Operation(23);
    pub const SET_INDEX: Operation = Operation(24);
}

impl Operation {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::NO_OP => "NO_OP",
            Self::MOVE => "MOVE",
            Self::DROP => "DROP",
            Self::ADD => "ADD",
            Self::SUBTRACT => "SUBTRACT",
            Self::MULTIPLY => "MULTIPLY",
            Self::DIVIDE => "DIVIDE",
            Self::MODULO => "MODULO",
            Self::POWER => "POWER",
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
            Self::CHECK_INDEX => "CHECK_INDEX",
            Self::GET_INDEX => "GET_INDEX",
            Self::SET_INDEX => "SET_INDEX",
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
