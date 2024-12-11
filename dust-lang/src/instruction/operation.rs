//! Part of an [Instruction][crate::Instruction] that is encoded as a single byte.

use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Serialize};

pub const MOVE_BYTE: u8 = 0;
pub const CLOSE_BYTE: u8 = 1;
pub const LOAD_BOOLEAN_BYTE: u8 = 2;
pub const LOAD_CONSTANT_BYTE: u8 = 3;
pub const LOAD_LIST_BYTE: u8 = 4;
pub const LOAD_SELF_BYTE: u8 = 5;
pub const GET_LOCAL_BYTE: u8 = 6;
pub const SET_LOCAL_BYTE: u8 = 7;
pub const ADD_BYTE: u8 = 8;
pub const SUBTRACT_BYTE: u8 = 9;
pub const MULTIPLY_BYTE: u8 = 10;
pub const DIVIDE_BYTE: u8 = 11;
pub const MODULO_BYTE: u8 = 12;
pub const TEST_BYTE: u8 = 13;
pub const TEST_SET_BYTE: u8 = 14;
pub const EQUAL_BYTE: u8 = 15;
pub const LESS_BYTE: u8 = 16;
pub const LESS_EQUAL_BYTE: u8 = 17;
pub const NEGATE_BYTE: u8 = 18;
pub const NOT_BYTE: u8 = 19;
pub const CALL_BYTE: u8 = 20;
pub const CALL_NATIVE_BYTE: u8 = 21;
pub const JUMP_BYTE: u8 = 22;
pub const RETURN_BYTE: u8 = 23;

/// Part of an [Instruction][crate::Instruction] that is encoded as a single byte.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Operation {
    Move = MOVE_BYTE,
    Close = CLOSE_BYTE,
    LoadBoolean = LOAD_BOOLEAN_BYTE,
    LoadConstant = LOAD_CONSTANT_BYTE,
    LoadList = LOAD_LIST_BYTE,
    LoadSelf = LOAD_SELF_BYTE,
    GetLocal = GET_LOCAL_BYTE,
    SetLocal = SET_LOCAL_BYTE,
    Add = ADD_BYTE,
    Subtract = SUBTRACT_BYTE,
    Multiply = MULTIPLY_BYTE,
    Divide = DIVIDE_BYTE,
    Modulo = MODULO_BYTE,
    Test = TEST_BYTE,
    TestSet = TEST_SET_BYTE,
    Equal = EQUAL_BYTE,
    Less = LESS_BYTE,
    LessEqual = LESS_EQUAL_BYTE,
    Negate = NEGATE_BYTE,
    Not = NOT_BYTE,
    Call = CALL_BYTE,
    CallNative = CALL_NATIVE_BYTE,
    Jump = JUMP_BYTE,
    Return = RETURN_BYTE,
}

impl From<u8> for Operation {
    fn from(byte: u8) -> Self {
        match byte {
            MOVE_BYTE => Self::Move,
            CLOSE_BYTE => Self::Close,
            LOAD_BOOLEAN_BYTE => Self::LoadBoolean,
            LOAD_CONSTANT_BYTE => Self::LoadConstant,
            LOAD_LIST_BYTE => Self::LoadList,
            LOAD_SELF_BYTE => Self::LoadSelf,
            GET_LOCAL_BYTE => Self::GetLocal,
            SET_LOCAL_BYTE => Self::SetLocal,
            ADD_BYTE => Self::Add,
            SUBTRACT_BYTE => Self::Subtract,
            MULTIPLY_BYTE => Self::Multiply,
            DIVIDE_BYTE => Self::Divide,
            MODULO_BYTE => Self::Modulo,
            TEST_BYTE => Self::Test,
            TEST_SET_BYTE => Self::TestSet,
            EQUAL_BYTE => Self::Equal,
            LESS_BYTE => Self::Less,
            LESS_EQUAL_BYTE => Self::LessEqual,
            NEGATE_BYTE => Self::Negate,
            NOT_BYTE => Self::Not,
            CALL_BYTE => Self::Call,
            CALL_NATIVE_BYTE => Self::CallNative,
            JUMP_BYTE => Self::Jump,
            RETURN_BYTE => Self::Return,
            _ => {
                if cfg!(debug_assertions) {
                    panic!("Invalid operation byte: {}", byte)
                } else {
                    Self::Return
                }
            }
        }
    }
}

impl Operation {
    pub fn name(self) -> &'static str {
        match self {
            Self::Move => "MOVE",
            Self::Close => "CLOSE",
            Self::LoadBoolean => "LOAD_BOOLEAN",
            Self::LoadConstant => "LOAD_CONSTANT",
            Self::LoadList => "LOAD_LIST",
            Self::LoadSelf => "LOAD_SELF",
            Self::GetLocal => "GET_LOCAL",
            Self::SetLocal => "SET_LOCAL",
            Self::Add => "ADD",
            Self::Subtract => "SUBTRACT",
            Self::Multiply => "MULTIPLY",
            Self::Divide => "DIVIDE",
            Self::Modulo => "MODULO",
            Self::Test => "TEST",
            Self::TestSet => "TEST_SET",
            Self::Equal => "EQUAL",
            Self::Less => "LESS",
            Self::LessEqual => "LESS_EQUAL",
            Self::Negate => "NEGATE",
            Self::Not => "NOT",
            Self::Call => "CALL",
            Self::CallNative => "CALL_NATIVE",
            Self::Jump => "JUMP",
            Self::Return => "RETURN",
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_OPERATIONS: [Operation; 24] = [
        Operation::Move,
        Operation::Close,
        Operation::LoadBoolean,
        Operation::LoadConstant,
        Operation::LoadList,
        Operation::LoadSelf,
        Operation::GetLocal,
        Operation::SetLocal,
        Operation::Add,
        Operation::Subtract,
        Operation::Multiply,
        Operation::Divide,
        Operation::Modulo,
        Operation::Test,
        Operation::TestSet,
        Operation::Equal,
        Operation::Less,
        Operation::LessEqual,
        Operation::Negate,
        Operation::Not,
        Operation::Call,
        Operation::CallNative,
        Operation::Jump,
        Operation::Return,
    ];

    #[test]
    fn operations_are_unique() {
        for (i, operation) in ALL_OPERATIONS.into_iter().enumerate() {
            assert_eq!(i, operation as usize);
        }
    }

    #[test]
    fn operation_uses_five_bits() {
        for operation in ALL_OPERATIONS {
            assert_eq!(operation as u8 & 0b1110_0000, 0);
        }
    }

    #[test]
    fn operation_is_one_byte() {
        assert_eq!(size_of::<Operation>(), 1);
    }
}
