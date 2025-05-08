//! Part of an [Instruction][crate::Instruction] that is encoded as a single byte.

use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

/// Part of an [Instruction][crate::Instruction] that is encoded as a single byte.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Operation(pub u8);

impl Operation {
    pub const NO_OP: Operation = Operation(0);

    // Memory and register manipulation
    pub const MOVE: Operation = Operation(1);
    pub const CLOSE: Operation = Operation(2);

    // Loaders
    pub const LOAD_ENCODED: Operation = Operation(3);
    pub const LOAD_CONSTANT: Operation = Operation(4);
    pub const LOAD_LIST: Operation = Operation(5);
    pub const LOAD_FUNCTION: Operation = Operation(6);

    // Arithmetic
    pub const ADD: Operation = Operation(7);
    pub const SUBTRACT: Operation = Operation(8);
    pub const MULTIPLY: Operation = Operation(9);
    pub const DIVIDE: Operation = Operation(10);
    pub const MODULO: Operation = Operation(11);

    // Comparison
    pub const EQUAL: Operation = Operation(12);
    pub const LESS: Operation = Operation(13);
    pub const LESS_EQUAL: Operation = Operation(14);

    // Unary operations
    pub const NEGATE: Operation = Operation(15);
    pub const NOT: Operation = Operation(16);

    // Logical operations
    pub const TEST: Operation = Operation(17);
    pub const TEST_SET: Operation = Operation(18);

    // Function calls
    pub const CALL: Operation = Operation(19);
    pub const CALL_NATIVE: Operation = Operation(20);

    // Control flow
    pub const JUMP: Operation = Operation(21);
    pub const RETURN: Operation = Operation(22);
}

impl Operation {
    pub fn name(&self) -> &'static str {
        match *self {
            Self::NO_OP => "NO_OP",
            Self::MOVE => "MOVE",
            Self::CLOSE => "CLOSE",
            Self::LOAD_ENCODED => "LOAD_ENCODED",
            Self::LOAD_CONSTANT => "LOAD_CONSTANT",
            Self::LOAD_FUNCTION => "LOAD_FUNCTION",
            Self::LOAD_LIST => "LOAD_LIST",
            Self::ADD => "ADD",
            Self::SUBTRACT => "SUBTRACT",
            Self::MULTIPLY => "MULTIPLY",
            Self::DIVIDE => "DIVIDE",
            Self::MODULO => "MODULO",
            Self::EQUAL => "EQUAL",
            Self::LESS => "LESS",
            Self::LESS_EQUAL => "LESS_EQUAL",
            Self::NEGATE => "NEGATE",
            Self::NOT => "NOT",
            Self::TEST => "TEST",
            Self::TEST_SET => "TEST_SET",
            Self::CALL => "CALL",
            Self::CALL_NATIVE => "CALL_NATIVE",
            Self::JUMP => "JUMP",
            Self::RETURN => "RETURN",
            unknown => panic!("Unknown operation: {}", unknown.0),
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
