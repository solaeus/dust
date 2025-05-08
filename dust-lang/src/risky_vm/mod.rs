//! An experimental Dust virtual machine that uses `unsafe` code. This VM never emits errors.
//! Instead, errors are handled as panics in debug mode but, in release mode, the use of `unsafe`
//! will cause undefined behavior.
mod runner;
mod thread;

use core::panicking::panic;
use std::{collections::HashMap, ops::RangeInclusive, sync::Arc, thread::Builder};

pub use thread::Thread;

use crossbeam_channel::bounded;
use tracing::{Level, span};

use crate::{
    AbstractList, Address, Chunk, DustError, DustString, Function, Type, TypeCode, Value, compile,
    instruction::AddressKind,
};

pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let chunk = compile(source)?;
    let vm = Vm::new(chunk);

    Ok(vm.run())
}

pub struct Vm {
    main_chunk: Chunk,
}

impl Vm {
    pub fn new(main_chunk: Chunk) -> Self {
        Self { main_chunk }
    }

    pub fn run(self) -> Option<Value> {
        let span = span!(Level::INFO, "Run");
        let _enter = span.enter();

        let thread_name = self
            .main_chunk
            .name
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or_else(|| "anonymous".to_string());
        let (tx, rx) = bounded(1);

        Builder::new()
            .name(thread_name)
            .spawn(move || {
                let main_chunk = Arc::new(self.main_chunk);
                let main_thread = Thread::new(main_chunk);
                let return_value = main_thread.run();
                let _ = tx.send(return_value);
            })
            .unwrap()
            .join()
            .unwrap();

        rx.recv().unwrap_or(None)
    }
}

#[derive(Clone, Debug)]
pub struct CallFrame {
    pub chunk: Arc<Chunk>,
    pub ip: usize,
    pub return_register: u16,
}

impl CallFrame {
    pub fn new(chunk: Arc<Chunk>, return_register: u16) -> Self {
        Self {
            chunk,
            ip: 0,
            return_register,
        }
    }

    pub fn get_character_constant(&self, constant_index: usize) -> char {
        if cfg!(debug_assertions) {
            *self.chunk.character_constants.get(constant_index).unwrap()
        } else {
            unsafe { *self.chunk.character_constants.get_unchecked(constant_index) }
        }
    }

    pub fn get_float_constant(&self, constant_index: usize) -> f64 {
        if cfg!(debug_assertions) {
            *self.chunk.float_constants.get(constant_index).unwrap()
        } else {
            unsafe { *self.chunk.float_constants.get_unchecked(constant_index) }
        }
    }

    pub fn get_integer_constant(&self, constant_index: usize) -> i64 {
        if cfg!(debug_assertions) {
            *self.chunk.integer_constants.get(constant_index).unwrap()
        } else {
            unsafe { *self.chunk.integer_constants.get_unchecked(constant_index) }
        }
    }

    pub fn get_string_constant(&self, constant_index: usize) -> &DustString {
        if cfg!(debug_assertions) {
            self.chunk.string_constants.get(constant_index).unwrap()
        } else {
            unsafe { self.chunk.string_constants.get_unchecked(constant_index) }
        }
    }

    pub fn get_argument_list(&self, index: u16) -> &(Vec<(u16, TypeCode)>, Vec<Type>) {
        if cfg!(debug_assertions) {
            self.chunk.argument_lists.get(index as usize).unwrap()
        } else {
            unsafe { self.chunk.argument_lists.get_unchecked(index as usize) }
        }
    }
}

#[derive(Debug)]
pub struct Memory {
    pub register_table: RegisterTable,
    pub heap_slot_table: HeapSlotTable,
}

#[derive(Debug)]
pub struct RegisterTable {
    pub booleans: RegisterList<bool>,
    pub bytes: RegisterList<u8>,
    pub characters: RegisterList<char>,
    pub floats: RegisterList<f64>,
    pub integers: RegisterList<i64>,
    pub strings: RegisterList<String>,
    pub lists: RegisterList<AbstractList>,
    pub functions: RegisterList<Function>,
}

impl Default for RegisterTable {
    fn default() -> Self {
        RegisterTable {
            booleans: RegisterList::default(),
            bytes: RegisterList::default(),
            characters: RegisterList::default(),
            floats: RegisterList::default(),
            integers: RegisterList::default(),
            strings: RegisterList::default(),
            lists: RegisterList::default(),
            functions: RegisterList::default(),
        }
    }
}

#[derive(Debug)]
pub struct RegisterList<T> {
    pub r_0: T,
    pub r_1: T,
    pub r_2: T,
    pub r_3: T,
    pub r_4: T,
    pub r_5: T,
    pub r_6: T,
    pub r_7: T,
    pub r_8: T,
    pub r_9: T,
}

impl<T: Default> Default for RegisterList<T> {
    fn default() -> Self {
        RegisterList {
            r_0: T::default(),
            r_1: T::default(),
            r_2: T::default(),
            r_3: T::default(),
            r_4: T::default(),
            r_5: T::default(),
            r_6: T::default(),
            r_7: T::default(),
            r_8: T::default(),
            r_9: T::default(),
        }
    }
}

#[derive(Debug)]
pub struct HeapSlotTable {
    pub booleans: HashMap<u16, HeapSlot<bool>>,
    pub bytes: HashMap<u16, HeapSlot<u8>>,
    pub characters: HashMap<u16, HeapSlot<char>>,
    pub floats: HashMap<u16, HeapSlot<f64>>,
    pub integers: HashMap<u16, HeapSlot<i64>>,
    pub strings: HashMap<u16, HeapSlot<DustString>>,
    pub lists: HashMap<u16, HeapSlot<AbstractList>>,
    pub functions: HashMap<u16, HeapSlot<Function>>,
}

impl HeapSlotTable {
    pub fn new(chunk: &Chunk) -> Self {
        Self {
            booleans: HashMap::with_capacity(chunk.boolean_memory_length as usize),
            bytes: HashMap::with_capacity(chunk.byte_memory_length as usize),
            characters: HashMap::with_capacity(chunk.character_memory_length as usize),
            floats: HashMap::with_capacity(chunk.function_memory_length as usize),
            integers: HashMap::with_capacity(chunk.integer_memory_length as usize),
            strings: HashMap::with_capacity(chunk.string_memory_length as usize),
            lists: HashMap::with_capacity(chunk.list_memory_length as usize),
            functions: HashMap::with_capacity(chunk.function_memory_length as usize),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HeapSlot<T> {
    value: T,
    is_closed: bool,
}

impl<T> HeapSlot<T> {
    pub fn new(value: T) -> Self {
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

impl<T: Copy> HeapSlot<T> {
    pub fn copy_value(&self) -> T {
        self.value
    }
}

impl<T: Clone> HeapSlot<T> {
    pub fn clone_value(&self) -> T {
        self.value.clone()
    }
}

impl<T: Default> Default for HeapSlot<T> {
    fn default() -> Self {
        Self {
            value: Default::default(),
            is_closed: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Pointer {
    Constant(u16),
    Memory(u16),
    Register(u16),
}

impl Pointer {
    pub fn index(&self) -> u16 {
        match self {
            Pointer::Constant(index) => *index,
            Pointer::Memory(index) => *index,
            Pointer::Register(index) => *index,
        }
    }
}
