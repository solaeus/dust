//! Part of an [Instruction][crate::Instruction] that is encoded as a single byte.

use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

/// Part of an [Instruction][crate::Instruction] that is encoded as a single byte.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Operation(pub u8);

impl Operation {
    // Stack manipulation
    pub const POINT: Operation = Operation(0);
    pub const CLOSE: Operation = Operation(1);

    // Loaders
    pub const LOAD_BOOLEAN: Operation = Operation(2);
    pub const LOAD_CONSTANT: Operation = Operation(3);
    pub const LOAD_FUNCTION: Operation = Operation(4);
    pub const LOAD_LIST: Operation = Operation(5);
    pub const LOAD_SELF: Operation = Operation(6);

    // Locals
    pub const GET_LOCAL: Operation = Operation(7);
    pub const SET_LOCAL: Operation = Operation(8);

    // Addition
    pub const ADD_INT: Operation = Operation(9);
    pub const ADD_FLOAT: Operation = Operation(10);
    pub const ADD_BYTE: Operation = Operation(11);
    pub const ADD_STR: Operation = Operation(12);
    pub const ADD_CHAR: Operation = Operation(13);
    pub const ADD_STR_CHAR: Operation = Operation(14);
    pub const ADD_CHAR_STR: Operation = Operation(15);

    // Subtraction
    pub const SUBTRACT_INT: Operation = Operation(16);
    pub const SUBTRACT_FLOAT: Operation = Operation(17);
    pub const SUBTRACT_BYTE: Operation = Operation(18);

    // Multiplication
    pub const MULTIPLY_INT: Operation = Operation(19);
    pub const MULTIPLY_FLOAT: Operation = Operation(20);
    pub const MULTIPLY_BYTE: Operation = Operation(21);

    // Division
    pub const DIVIDE_INT: Operation = Operation(22);
    pub const DIVIDE_FLOAT: Operation = Operation(23);
    pub const DIVIDE_BYTE: Operation = Operation(24);

    // Modulo
    pub const MODULO_INT: Operation = Operation(25);
    pub const MODULO_FLOAT: Operation = Operation(26);
    pub const MODULO_BYTE: Operation = Operation(27);

    // Equality
    pub const EQUAL_INT: Operation = Operation(28);
    pub const EQUAL_FLOAT: Operation = Operation(29);
    pub const EQUAL_BYTE: Operation = Operation(30);
    pub const EQUAL_STR: Operation = Operation(31);
    pub const EQUAL_CHAR: Operation = Operation(32);
    pub const EQUAL_STR_CHAR: Operation = Operation(33);
    pub const EQUAL_CHAR_STR: Operation = Operation(34);
    pub const EQUAL_BOOL: Operation = Operation(35);

    // < or >= comparison
    pub const LESS_INT: Operation = Operation(36);
    pub const LESS_FLOAT: Operation = Operation(37);
    pub const LESS_BYTE: Operation = Operation(38);
    pub const LESS_STR: Operation = Operation(39);
    pub const LESS_CHAR: Operation = Operation(40);

    // <= or > comparison
    pub const LESS_EQUAL_INT: Operation = Operation(41);
    pub const LESS_EQUAL_FLOAT: Operation = Operation(42);
    pub const LESS_EQUAL_BYTE: Operation = Operation(43);
    pub const LESS_EQUAL_STR: Operation = Operation(44);
    pub const LESS_EQUAL_CHAR: Operation = Operation(45);

    // Unary operations
    pub const NEGATE_INT: Operation = Operation(46);
    pub const NEGATE_FLOAT: Operation = Operation(47);
    pub const NOT: Operation = Operation(48);

    // Logical operations
    pub const TEST: Operation = Operation(49);
    pub const TEST_SET: Operation = Operation(50);

    // Function calls
    pub const CALL: Operation = Operation(51);
    pub const CALL_NATIVE: Operation = Operation(52);

    // Control flow
    pub const JUMP: Operation = Operation(53);
    pub const RETURN: Operation = Operation(54);
}

impl Operation {
    pub fn name(&self) -> &'static str {
        match *self {
            Self::POINT => "POINT",
            Self::CLOSE => "CLOSE",
            Self::LOAD_BOOLEAN => "LOAD_BOOLEAN",
            Self::LOAD_CONSTANT => "LOAD_CONSTANT",
            Self::LOAD_FUNCTION => "LOAD_FUNCTION",
            Self::LOAD_LIST => "LOAD_LIST",
            Self::LOAD_SELF => "LOAD_SELF",
            Self::GET_LOCAL => "GET_LOCAL",
            Self::SET_LOCAL => "SET_LOCAL",
            Self::ADD_INT => "ADD_INT",
            Self::ADD_FLOAT => "ADD_FLOAT",
            Self::ADD_BYTE => "ADD_BYTE",
            Self::ADD_STR => "ADD_STR",
            Self::ADD_CHAR => "ADD_CHAR",
            Self::ADD_STR_CHAR => "ADD_STR_CHAR",
            Self::ADD_CHAR_STR => "ADD_CHAR_STR",
            Self::SUBTRACT_INT => "SUBTRACT_INT",
            Self::SUBTRACT_FLOAT => "SUBTRACT_FLOAT",
            Self::SUBTRACT_BYTE => "SUBTRACT_BYTE",
            Self::MULTIPLY_INT => "MULTIPLY_INT",
            Self::MULTIPLY_FLOAT => "MULTIPLY_FLOAT",
            Self::MULTIPLY_BYTE => "MULTIPLY_BYTE",
            Self::DIVIDE_INT => "DIVIDE_INT",
            Self::DIVIDE_FLOAT => "DIVIDE_FLOAT",
            Self::DIVIDE_BYTE => "DIVIDE_BYTE",
            Self::MODULO_INT => "MODULO_INT",
            Self::MODULO_FLOAT => "MODULO_FLOAT",
            Self::MODULO_BYTE => "MODULO_BYTE",
            Self::EQUAL_INT => "EQUAL_INT",
            Self::EQUAL_FLOAT => "EQUAL_FLOAT",
            Self::EQUAL_BYTE => "EQUAL_BYTE",
            Self::EQUAL_STR => "EQUAL_STR",
            Self::EQUAL_CHAR => "EQUAL_CHAR",
            Self::EQUAL_STR_CHAR => "EQUAL_STR_CHAR",
            Self::EQUAL_CHAR_STR => "EQUAL_CHAR_STR",
            Self::EQUAL_BOOL => "EQUAL_BOOL",
            Self::LESS_INT => "LESS_INT",
            Self::LESS_FLOAT => "LESS_FLOAT",
            Self::LESS_BYTE => "LESS_BYTE",
            Self::LESS_STR => "LESS_STR",
            Self::LESS_CHAR => "LESS_CHAR",
            Self::LESS_EQUAL_INT => "LESS_EQUAL_INT",
            Self::LESS_EQUAL_FLOAT => "LESS_EQUAL_FLOAT",
            Self::LESS_EQUAL_BYTE => "LESS_EQUAL_BYTE",
            Self::LESS_EQUAL_STR => "LESS_EQUAL_STR",
            Self::LESS_EQUAL_CHAR => "LESS_EQUAL_CHAR",
            Self::NEGATE_INT => "NEGATE_INT",
            Self::NEGATE_FLOAT => "NEGATE_FLOAT",
            Self::NOT => "NOT",
            Self::TEST => "TEST",
            Self::TEST_SET => "TEST_SET",
            Self::CALL => "CALL",
            Self::CALL_NATIVE => "CALL_NATIVE",
            Self::JUMP => "JUMP",
            Self::RETURN => "RETURN",
            _ => Self::panic_from_unknown_code(self.0),
        }
    }

    pub fn is_math(self) -> bool {
        matches!(
            self,
            Operation::ADD_INT
                | Operation::ADD_FLOAT
                | Operation::ADD_BYTE
                | Operation::SUBTRACT_INT
                | Operation::SUBTRACT_FLOAT
                | Operation::SUBTRACT_BYTE
                | Operation::MULTIPLY_INT
                | Operation::MULTIPLY_FLOAT
                | Operation::MULTIPLY_BYTE
                | Operation::DIVIDE_INT
                | Operation::DIVIDE_FLOAT
                | Operation::DIVIDE_BYTE
                | Operation::MODULO_INT
                | Operation::MODULO_FLOAT
                | Operation::MODULO_BYTE
        )
    }

    pub fn is_comparison(self) -> bool {
        matches!(
            self,
            Operation::EQUAL_INT
                | Operation::EQUAL_FLOAT
                | Operation::EQUAL_BYTE
                | Operation::EQUAL_STR
                | Operation::EQUAL_CHAR
                | Operation::EQUAL_STR_CHAR
                | Operation::EQUAL_CHAR_STR
                | Operation::EQUAL_BOOL
                | Operation::LESS_INT
                | Operation::LESS_FLOAT
                | Operation::LESS_BYTE
                | Operation::LESS_STR
                | Operation::LESS_CHAR
                | Operation::LESS_EQUAL_INT
                | Operation::LESS_EQUAL_FLOAT
                | Operation::LESS_EQUAL_BYTE
                | Operation::LESS_EQUAL_STR
                | Operation::LESS_EQUAL_CHAR
        )
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
