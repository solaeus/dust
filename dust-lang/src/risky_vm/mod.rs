//! An experimental Dust virtual machine that uses `unsafe` code. This VM never emits errors.
//! Instead, errors are handled as panics in debug mode but, in release mode, the use of `unsafe`
//! will cause undefined behavior.
mod thread;

use std::{ops::RangeInclusive, sync::Arc, thread::Builder};

pub use thread::Thread;

use crossbeam_channel::bounded;
use tracing::{Level, span};

use crate::{AbstractList, Chunk, DustError, DustString, Function, Type, TypeCode, Value, compile};

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
pub struct Registers {
    pub booleans: RegisterList<bool>,
    pub bytes: RegisterList<u8>,
    pub characters: RegisterList<char>,
    pub floats: RegisterList<f64>,
    pub integers: RegisterList<i64>,
    pub strings: RegisterList<String>,
    pub lists: RegisterList<AbstractList>,
    pub functions: RegisterList<Function>,
}

impl Default for Registers {
    fn default() -> Self {
        Registers {
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
pub struct MemoryHeap {
    pub booleans: HeapSlotList<bool>,
    pub bytes: HeapSlotList<u8>,
    pub characters: HeapSlotList<char>,
    pub floats: HeapSlotList<f64>,
    pub integers: HeapSlotList<i64>,
    pub strings: HeapSlotList<DustString>,
    pub lists: HeapSlotList<AbstractList>,
    pub functions: HeapSlotList<Function>,
}

impl MemoryHeap {
    pub fn new(chunk: &Chunk) -> Self {
        Self {
            booleans: HeapSlotList::new(chunk.boolean_register_count as usize),
            bytes: HeapSlotList::new(chunk.byte_register_count as usize),
            characters: HeapSlotList::new(chunk.character_register_count as usize),
            floats: HeapSlotList::new(chunk.float_register_count as usize),
            integers: HeapSlotList::new(chunk.integer_register_count as usize),
            strings: HeapSlotList::new(chunk.string_register_count as usize),
            lists: HeapSlotList::new(chunk.list_register_count as usize),
            functions: HeapSlotList::new(chunk.function_register_count as usize),
        }
    }
}

#[derive(Debug)]
pub struct HeapSlotList<T, const STACK_LEN: usize = 64> {
    pub registers: Vec<HeapSlot<T>>,
}

impl<T, const STACK_LEN: usize> HeapSlotList<T, STACK_LEN>
where
    T: Clone + Default,
{
    pub fn new(length: usize) -> Self {
        let mut registers = Vec::with_capacity(length);

        for _ in 0..length {
            registers.push(HeapSlot::default());
        }

        Self { registers }
    }

    pub fn get(&self, index: usize) -> &HeapSlot<T> {
        if cfg!(debug_assertions) {
            self.registers.get(index).unwrap()
        } else {
            unsafe { self.registers.get_unchecked(index) }
        }
    }

    pub fn get_many_mut(&mut self, indices: RangeInclusive<usize>) -> &mut [HeapSlot<T>] {
        let registers = if cfg!(debug_assertions) {
            self.registers.get_disjoint_mut([indices]).unwrap()
        } else {
            unsafe { self.registers.get_disjoint_unchecked_mut([indices]) }
        };

        registers[0]
    }

    pub fn get_mut(&mut self, index: usize) -> &mut HeapSlot<T> {
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

        self.registers[index] = HeapSlot::new(new_value)
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
