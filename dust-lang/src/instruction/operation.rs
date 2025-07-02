//! Part of an [Instruction][crate::Instruction] that is encoded as a single byte.

use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

/// Part of an [Instruction][crate::Instruction] that is encoded as a single byte.
#[derive(Clone, Copy, Default, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Operation(pub u8);

impl Operation {
    pub const NO_OP: Operation = Operation(0);

    // Memory manipulation
    pub const LOAD: Operation = Operation(1);
    pub const LIST: Operation = Operation(2);

    // Arithmetic binary operations
    pub const ADD: Operation = Operation(3);
    pub const SUBTRACT: Operation = Operation(4);
    pub const MULTIPLY: Operation = Operation(5);
    pub const DIVIDE: Operation = Operation(6);
    pub const MODULO: Operation = Operation(7);

    // Comparison binary operations
    pub const EQUAL: Operation = Operation(8);
    pub const LESS: Operation = Operation(9);
    pub const LESS_EQUAL: Operation = Operation(10);

    // Logical AND/OR operations
    pub const TEST: Operation = Operation(11);

    // Unary numeric negation and logical NOT
    pub const NEGATE: Operation = Operation(12);

    // Function calls
    pub const CALL: Operation = Operation(13);
    pub const CALL_NATIVE: Operation = Operation(14);

    // Control flow
    pub const JUMP: Operation = Operation(15);
    pub const RETURN: Operation = Operation(16);
}

impl Operation {
    pub fn name(&self) -> &'static str {
        match *self {
            Self::NO_OP => "NO_OP",
            Self::LOAD => "LOAD",
            Self::LIST => "LIST",
            Self::ADD => "ADD",
            Self::SUBTRACT => "SUBTRACT",
            Self::MULTIPLY => "MULTIPLY",
            Self::DIVIDE => "DIVIDE",
            Self::MODULO => "MODULO",
            Self::EQUAL => "EQUAL",
            Self::LESS => "LESS",
            Self::LESS_EQUAL => "LESS_EQUAL",
            Self::NEGATE => "NEGATE",
            Self::TEST => "TEST",
            Self::CALL => "CALL",
            Self::CALL_NATIVE => "CALL_NATIVE",
            Self::JUMP => "JUMP",
            Self::RETURN => "RETURN",
            _ => "UNKNOWN",
        }
    }

    pub fn is_math(self) -> bool {
        matches!(
            self,
            Operation::ADD
                | Operation::SUBTRACT
                | Operation::MULTIPLY
                | Operation::DIVIDE
                | Operation::MODULO
        )
    }

    pub fn is_comparison(self) -> bool {
        matches!(
            self,
            Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL
        )
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
