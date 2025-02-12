use std::{
    fmt::{self, Debug, Display, Formatter},
    ptr,
    rc::Rc,
};

use smallvec::{SmallVec, smallvec};

use crate::{AbstractList, Chunk, DustString, Function};

#[derive(Debug)]
pub struct CallFrame {
    pub chunk: Rc<Chunk>,
    pub ip: usize,
    pub return_register: u16,
    pub registers: RegisterTable,
}

impl CallFrame {
    pub fn new(chunk: Rc<Chunk>, return_register: u16) -> Self {
        let registers = RegisterTable::new();

        Self {
            chunk,
            ip: 0,
            return_register,
            registers,
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
            booleans: smallvec![Register::default(); 64],
            bytes: smallvec![Register::default(); 64],
            characters: smallvec![Register::default(); 64],
            floats: smallvec![Register::default(); 64],
            integers: smallvec![Register::default(); 64],
            strings: smallvec![Register::default(); 64],
            lists: smallvec![Register::default(); 64],
            functions: smallvec![Register::default(); 64],
        }
    }
}

impl Default for RegisterTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Register<T: Default> {
    Value(T),
    Closed(T),
    Pointer(Pointer),
}

impl<T: Default> Register<T> {
    pub fn contained_value_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Value(value) => Some(value),
            Self::Closed(value) => Some(value),
            Self::Pointer(_) => None,
        }
    }
}

impl<T: Default> Default for Register<T> {
    fn default() -> Self {
        Self::Value(T::default())
    }
}

impl<T: Default + Display> Display for Register<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Closed(value) => write!(f, "Closed({value})"),
            Self::Value(value) => write!(f, "Value({value})"),
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

#[derive(Debug, Clone, Copy)]
pub struct PointerCache {
    pub integer_mut: *mut i64,
    pub integer_left: *const i64,
    pub integer_right: *const i64,
}

impl PointerCache {
    pub fn new() -> Self {
        Self {
            integer_mut: ptr::null_mut(),
            integer_left: ptr::null(),
            integer_right: ptr::null(),
        }
    }
}

impl Default for PointerCache {
    fn default() -> Self {
        Self::new()
    }
}
