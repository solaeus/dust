use std::{
    fmt::{self, Debug, Display, Formatter},
    ops::{Index, IndexMut, RangeInclusive},
    sync::Arc,
};

use smallvec::SmallVec;

use crate::{AbstractList, Chunk, DustString, Function};

#[derive(Debug)]
pub struct CallFrame {
    pub chunk: Arc<Chunk>,
    pub ip: usize,
    pub return_register: u16,
    pub registers: RegisterTable,
}

impl CallFrame {
    pub fn new(chunk: Arc<Chunk>, return_register: u16) -> Self {
        let registers = RegisterTable {
            booleans: RegisterList::new(chunk.boolean_register_count as usize),
            bytes: RegisterList::new(chunk.byte_register_count as usize),
            characters: RegisterList::new(chunk.character_register_count as usize),
            floats: RegisterList::new(chunk.float_register_count as usize),
            integers: RegisterList::new(chunk.integer_register_count as usize),
            strings: RegisterList::new(chunk.string_register_count as usize),
            lists: RegisterList::new(chunk.list_register_count as usize),
            functions: RegisterList::new(chunk.function_register_count as usize),
        };

        Self {
            chunk,
            ip: 0,
            return_register,
            registers,
        }
    }

    pub fn get_boolean_from_pointer(&self, pointer: Pointer) -> bool {
        match pointer {
            Pointer::Register(register_index) => {
                *self.registers.booleans.get(register_index).as_value()
            }
            Pointer::Constant(_) => panic!("Attempted to get boolean from constant pointer"),
        }
    }

    pub fn get_byte_from_pointer(&self, pointer: Pointer) -> u8 {
        match pointer {
            Pointer::Register(register_index) => {
                *self.registers.bytes.get(register_index).as_value()
            }
            Pointer::Constant(_) => panic!("Attempted to get byte from constant pointer"),
        }
    }

    pub fn get_character_from_pointer(&self, pointer: Pointer) -> char {
        match pointer {
            Pointer::Register(register_index) => {
                *self.registers.characters.get(register_index).as_value()
            }
            Pointer::Constant(constant_index) => self.get_character_constant(constant_index),
        }
    }

    pub fn get_character_constant(&self, constant_index: usize) -> char {
        if cfg!(debug_assertions) {
            *self.chunk.character_constants.get(constant_index).unwrap()
        } else {
            unsafe { *self.chunk.character_constants.get_unchecked(constant_index) }
        }
    }

    pub fn get_float_from_pointer(&self, pointer: Pointer) -> f64 {
        match pointer {
            Pointer::Register(register_index) => {
                *self.registers.floats.get(register_index).as_value()
            }
            Pointer::Constant(constant_index) => self.get_float_constant(constant_index),
        }
    }

    pub fn get_float_constant(&self, constant_index: usize) -> f64 {
        if cfg!(debug_assertions) {
            *self.chunk.float_constants.get(constant_index).unwrap()
        } else {
            unsafe { *self.chunk.float_constants.get_unchecked(constant_index) }
        }
    }

    pub fn get_integer_from_pointer(&self, pointer: Pointer) -> i64 {
        match pointer {
            Pointer::Register(register_index) => {
                *self.registers.integers.get(register_index).as_value()
            }
            Pointer::Constant(constant_index) => self.get_integer_constant(constant_index),
        }
    }

    pub fn get_integer_constant(&self, constant_index: usize) -> i64 {
        if cfg!(debug_assertions) {
            *self.chunk.integer_constants.get(constant_index).unwrap()
        } else {
            unsafe { *self.chunk.integer_constants.get_unchecked(constant_index) }
        }
    }

    pub fn get_string_from_pointer(&self, pointer: Pointer) -> &DustString {
        match pointer {
            Pointer::Register(register_index) => {
                self.registers.strings.get(register_index).as_value()
            }
            Pointer::Constant(constant_index) => self.get_string_constant(constant_index),
        }
    }

    pub fn get_string_constant(&self, constant_index: usize) -> &DustString {
        if cfg!(debug_assertions) {
            self.chunk.string_constants.get(constant_index).unwrap()
        } else {
            unsafe { self.chunk.string_constants.get_unchecked(constant_index) }
        }
    }

    pub fn get_list_from_pointer(&self, pointer: &Pointer) -> &AbstractList {
        match pointer {
            Pointer::Register(register_index) => {
                self.registers.lists.get(*register_index).as_value()
            }
            Pointer::Constant(_) => panic!("Attempted to get list from constant pointer"),
        }
    }

    pub fn get_function_from_pointer(&self, pointer: &Pointer) -> &Function {
        match pointer {
            Pointer::Register(register_index) => {
                self.registers.functions.get(*register_index).as_value()
            }
            Pointer::Constant(_) => panic!("Attempted to get function from constant pointer"),
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
    pub booleans: RegisterList<bool>,
    pub bytes: RegisterList<u8>,
    pub characters: RegisterList<char>,
    pub floats: RegisterList<f64>,
    pub integers: RegisterList<i64>,
    pub strings: RegisterList<DustString>,
    pub lists: RegisterList<AbstractList>,
    pub functions: RegisterList<Function>,
}

#[derive(Debug)]
pub struct RegisterList<T, const STACK_LEN: usize = 64> {
    pub registers: SmallVec<[Register<T>; STACK_LEN]>,
}

impl<T, const STACK_LEN: usize> RegisterList<T, STACK_LEN>
where
    T: Clone + Default,
{
    pub fn new(length: usize) -> Self {
        let mut registers = SmallVec::with_capacity(length);

        for _ in 0..length {
            registers.push(Register::default());
        }

        Self { registers }
    }

    pub fn get(&self, index: usize) -> &Register<T> {
        if cfg!(debug_assertions) {
            self.registers.get(index).unwrap()
        } else {
            unsafe { self.registers.get_unchecked(index) }
        }
    }

    pub fn get_many_mut(&mut self, indices: RangeInclusive<usize>) -> &mut [Register<T>] {
        let registers = if cfg!(debug_assertions) {
            self.registers.get_disjoint_mut([indices]).unwrap()
        } else {
            unsafe { self.registers.get_disjoint_unchecked_mut([indices]) }
        };

        registers[0]
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Register<T> {
        if cfg!(debug_assertions) {
            let length = self.registers.len();

            self.registers
                .get_mut(index)
                .unwrap_or_else(|| panic!("Index out of bounds: {index}. Length is {length}"))
        } else {
            unsafe { self.registers.get_unchecked_mut(index) }
        }
    }

    pub fn set_to_new_register(&mut self, index: usize, new_value: T) {
        assert!(index < self.registers.len(), "Register index out of bounds");

        self.registers[index] = Register::value(new_value)
    }

    pub fn close(&mut self, index: usize) {
        if cfg!(debug_assertions) {
            self.registers.get_mut(index).unwrap().close()
        } else {
            unsafe { self.registers.get_unchecked_mut(index).close() }
        }
    }

    pub fn is_closed(&self, index: usize) -> bool {
        if cfg!(debug_assertions) {
            self.registers.get(index).unwrap().is_closed()
        } else {
            unsafe { self.registers.get_unchecked(index).is_closed() }
        }
    }
}

impl<T> Index<usize> for RegisterList<T> {
    type Output = Register<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.registers[index]
    }
}

impl<T> IndexMut<usize> for RegisterList<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.registers[index]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Register<T> {
    value: T,
    is_closed: bool,
}

impl<T> Register<T> {
    pub fn value(value: T) -> Self {
        Self {
            value,
            is_closed: false,
        }
    }

    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    pub fn close(&mut self) {
        self.is_closed = true;
    }

    pub fn set(&mut self, new_value: T) {
        self.value = new_value;
    }

    pub fn as_value(&self) -> &T {
        &self.value
    }

    pub fn as_value_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: Copy> Register<T> {
    pub fn copy_value(&self) -> T {
        self.value
    }
}

impl<T: Clone> Register<T> {
    pub fn clone_value(&self) -> T {
        self.value.clone()
    }
}

impl<T: Default> Default for Register<T> {
    fn default() -> Self {
        Self {
            value: Default::default(),
            is_closed: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Pointer {
    Register(usize),
    Constant(usize),
}
