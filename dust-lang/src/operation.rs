use std::fmt::{self, Display, Formatter};

const MOVE: u8 = 0b0000_0000;
const CLOSE: u8 = 0b000_0001;

const LOAD_BOOLEAN: u8 = 0b0000_0010;
const LOAD_CONSTANT: u8 = 0b0000_0011;
const LOAD_LIST: u8 = 0b0000_0100;

const DECLARE_LOCAL: u8 = 0b0000_0101;
const GET_LOCAL: u8 = 0b0000_0110;
const SET_LOCAL: u8 = 0b0000_0111;

const ADD: u8 = 0b0000_1000;
const SUBTRACT: u8 = 0b0000_1001;
const MULTIPLY: u8 = 0b0000_1010;
const DIVIDE: u8 = 0b0000_1011;
const MODULO: u8 = 0b0000_1100;

const TEST: u8 = 0b0000_1101;
const TEST_SET: u8 = 0b0000_1110;

const EQUAL: u8 = 0b0000_1111;
const LESS: u8 = 0b0001_0000;
const LESS_EQUAL: u8 = 0b0001_0001;

const NEGATE: u8 = 0b0001_0010;
const NOT: u8 = 0b0001_0011;

const JUMP: u8 = 0b0001_0100;
const RETURN: u8 = 0b0001_0101;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operation {
    // Stack manipulation
    Move = MOVE as isize,
    Close = CLOSE as isize,

    // Value loading
    LoadBoolean = LOAD_BOOLEAN as isize,
    LoadConstant = LOAD_CONSTANT as isize,
    LoadList = LOAD_LIST as isize,

    // Variables
    DefineLocal = DECLARE_LOCAL as isize,
    GetLocal = GET_LOCAL as isize,
    SetLocal = SET_LOCAL as isize,

    // Binary operations
    Add = ADD as isize,
    Subtract = SUBTRACT as isize,
    Multiply = MULTIPLY as isize,
    Divide = DIVIDE as isize,
    Modulo = MODULO as isize,

    // Logical operations
    Test = TEST as isize,
    TestSet = TEST_SET as isize,

    // Relational operations
    Equal = EQUAL as isize,
    Less = LESS as isize,
    LessEqual = LESS_EQUAL as isize,

    // Unary operations
    Negate = NEGATE as isize,
    Not = NOT as isize,

    // Control flow
    Jump = JUMP as isize,
    Return = RETURN as isize,
}

impl Operation {
    pub fn is_binary(&self) -> bool {
        matches!(
            self,
            Operation::Add
                | Operation::Subtract
                | Operation::Multiply
                | Operation::Divide
                | Operation::Modulo
        )
    }
}

impl From<u8> for Operation {
    fn from(byte: u8) -> Self {
        match byte {
            MOVE => Operation::Move,
            CLOSE => Operation::Close,
            LOAD_BOOLEAN => Operation::LoadBoolean,
            LOAD_CONSTANT => Operation::LoadConstant,
            LOAD_LIST => Operation::LoadList,
            DECLARE_LOCAL => Operation::DefineLocal,
            GET_LOCAL => Operation::GetLocal,
            SET_LOCAL => Operation::SetLocal,
            ADD => Operation::Add,
            SUBTRACT => Operation::Subtract,
            MULTIPLY => Operation::Multiply,
            DIVIDE => Operation::Divide,
            MODULO => Operation::Modulo,
            TEST => Operation::Test,
            TEST_SET => Operation::TestSet,
            EQUAL => Operation::Equal,
            LESS => Operation::Less,
            LESS_EQUAL => Operation::LessEqual,
            NEGATE => Operation::Negate,
            NOT => Operation::Not,
            JUMP => Operation::Jump,
            RETURN => Operation::Return,
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
            Operation::Move => MOVE,
            Operation::Close => CLOSE,
            Operation::LoadBoolean => LOAD_BOOLEAN,
            Operation::LoadConstant => LOAD_CONSTANT,
            Operation::LoadList => LOAD_LIST,
            Operation::DefineLocal => DECLARE_LOCAL,
            Operation::GetLocal => GET_LOCAL,
            Operation::SetLocal => SET_LOCAL,
            Operation::Add => ADD,
            Operation::Subtract => SUBTRACT,
            Operation::Multiply => MULTIPLY,
            Operation::Divide => DIVIDE,
            Operation::Modulo => MODULO,
            Operation::Test => TEST,
            Operation::TestSet => TEST_SET,
            Operation::Equal => EQUAL,
            Operation::Less => LESS,
            Operation::LessEqual => LESS_EQUAL,
            Operation::Negate => NEGATE,
            Operation::Not => NOT,
            Operation::Jump => JUMP,
            Operation::Return => RETURN,
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Operation::Move => write!(f, "MOVE"),
            Operation::Close => write!(f, "CLOSE"),
            Operation::LoadBoolean => write!(f, "LOAD_BOOLEAN"),
            Operation::LoadConstant => write!(f, "LOAD_CONSTANT"),
            Operation::LoadList => write!(f, "LOAD_LIST"),
            Operation::DefineLocal => write!(f, "DEFINE_LOCAL"),
            Operation::GetLocal => write!(f, "GET_LOCAL"),
            Operation::SetLocal => write!(f, "SET_LOCAL"),
            Operation::Add => write!(f, "ADD"),
            Operation::Subtract => write!(f, "SUBTRACT"),
            Operation::Multiply => write!(f, "MULTIPLY"),
            Operation::Divide => write!(f, "DIVIDE"),
            Operation::Modulo => write!(f, "MODULO"),
            Operation::Test => write!(f, "TEST"),
            Operation::TestSet => write!(f, "TEST_SET"),
            Operation::Equal => write!(f, "EQUAL"),
            Operation::Less => write!(f, "LESS"),
            Operation::LessEqual => write!(f, "LESS_EQUAL"),
            Operation::Negate => write!(f, "NEGATE"),
            Operation::Not => write!(f, "NOT"),
            Operation::Jump => write!(f, "JUMP"),
            Operation::Return => write!(f, "RETURN"),
        }
    }
}
