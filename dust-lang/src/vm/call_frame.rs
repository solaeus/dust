use std::{
    fmt::{self, Debug, Display, Formatter},
    sync::Arc,
};

use smallvec::{SmallVec, smallvec};

use crate::{AbstractList, Chunk, DustString, Function};

use super::action::ActionSequence;

#[derive(Debug)]
pub struct CallFrame {
    pub chunk: Arc<Chunk>,
    pub ip: usize,
    pub return_register: u16,
    pub registers: RegisterTable,
    pub action_sequence: ActionSequence,
}

impl CallFrame {
    pub fn new(chunk: Arc<Chunk>, return_register: u16) -> Self {
        let registers = RegisterTable::new();
        let action_sequence = ActionSequence::new(&chunk.instructions);

        Self {
            chunk,
            ip: 0,
            return_register,
            registers,
            action_sequence,
        }
    }
}

impl Display for CallFrame {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "FunctionCall: {} | IP: {}",
            self.chunk
                .name
                .as_ref()
                .unwrap_or(&DustString::from("anonymous")),
            self.ip,
        )
    }
}

#[derive(Debug)]
pub struct RegisterTable {
    pub booleans: SmallVec<[Register<bool>; 64]>,
    pub bytes: SmallVec<[Register<u8>; 64]>,
    pub characters: SmallVec<[Register<char>; 64]>,
    pub floats: SmallVec<[Register<f64>; 64]>,
    pub integers: SmallVec<[Register<i64>; 64]>,
    pub strings: SmallVec<[Register<DustString>; 64]>,
    pub lists: SmallVec<[Register<AbstractList>; 64]>,
    pub functions: SmallVec<[Register<Function>; 64]>,
}

impl RegisterTable {
    pub fn new() -> Self {
        Self {
            booleans: smallvec![Register::Empty; 64],
            bytes: smallvec![Register::Empty; 64],
            characters: smallvec![Register::Empty; 64],
            floats: smallvec![Register::Empty; 64],
            integers: smallvec![Register::Empty; 64],
            strings: smallvec![Register::Empty; 64],
            lists: smallvec![Register::Empty; 64],
            functions: smallvec![Register::Empty; 64],
        }
    }
}

impl Default for RegisterTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Register<T> {
    Empty,
    Value(T),
    Pointer(Pointer),
}

impl<T: Display> Display for Register<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty"),
            Self::Value(value) => write!(f, "{value}"),
            Self::Pointer(pointer) => write!(f, "Pointer({pointer:?})"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Pointer {
    RegisterBoolean(usize),
    RegisterByte(usize),
    RegisterCharacter(usize),
    RegisterFloat(usize),
    RegisterInteger(usize),
    RegisterString(usize),
    RegisterList(usize),
    RegisterFunction(usize),
    ConstantCharacter(usize),
    ConstantFloat(usize),
    ConstantInteger(usize),
    ConstantString(usize),
}

impl Display for Pointer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::RegisterBoolean(index) => write!(f, "P_R_BOOL_{index}"),
            Self::RegisterByte(index) => write!(f, "P_R_BYTE_{index}"),
            Self::RegisterCharacter(index) => write!(f, "P_R_CHAR_{index}"),
            Self::RegisterFloat(index) => write!(f, "P_R_FLOAT_{index}"),
            Self::RegisterInteger(index) => write!(f, "P_R_INT_{index}"),
            Self::RegisterString(index) => write!(f, "P_R_STR_{index}"),
            Self::RegisterList(index) => write!(f, "P_R_LIST_{index}"),
            Self::RegisterFunction(index) => write!(f, "P_R_FN_{index}"),
            Self::ConstantCharacter(index) => write!(f, "P_C_CHAR_{index}"),
            Self::ConstantFloat(index) => write!(f, "P_C_FLOAT_{index}"),
            Self::ConstantInteger(index) => write!(f, "P_C_INT_{index}"),
            Self::ConstantString(index) => write!(f, "P_C_STR_{index}"),
        }
    }
}
