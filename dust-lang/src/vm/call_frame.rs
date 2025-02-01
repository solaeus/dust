use std::{
    fmt::{self, Debug, Display, Formatter},
    sync::Arc,
};

use smallvec::{SmallVec, smallvec};

use crate::{Chunk, DustString, instruction::TypeCode};

#[derive(Debug)]
pub struct CallFrame {
    pub chunk: Arc<Chunk>,
    pub ip: usize,
    pub return_register: u16,
    pub registers: RegisterTable,
}

impl CallFrame {
    pub fn new(chunk: Arc<Chunk>, return_register: u16) -> Self {
        let register_count = chunk.register_count;

        Self {
            chunk,
            ip: 0,
            return_register,
            registers: RegisterTable::new(),
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
        }
    }
}

#[derive(Debug, Clone)]
pub enum Register<T> {
    Empty,
    Value(T),
    Pointer(Pointer),
}

#[derive(Debug, Clone, Copy)]
pub enum Pointer {
    Register(usize),
    Constant(usize),
}
