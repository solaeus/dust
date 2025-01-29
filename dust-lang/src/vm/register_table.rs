use core::slice::GetManyMutIndex;
use std::slice::SliceIndex;

use smallvec::{SmallVec, smallvec};
use tracing::trace;

use crate::{AbstractList, DustString};

#[derive(Debug, Clone)]
pub enum Register<T: Clone> {
    Empty,
    Value(T),
    Pointer(*mut Register<T>),
}

impl<T: Clone> Register<T> {
    pub fn expect_value(&self) -> &T {
        if let Self::Value(value) = self {
            value
        } else {
            panic!("Expected a value")
        }
    }

    pub fn expect_value_mut(&mut self) -> &mut T {
        if let Self::Value(value) = self {
            value
        } else {
            panic!("Expected a value")
        }
    }
}

const BOOLEAN_REGISTER_COUNT: usize = 64;
const BYTE_REGISTER_COUNT: usize = 64;
const CHARACTER_REGISTER_COUNT: usize = 64;
const FLOAT_REGISTER_COUNT: usize = 64;
const INTEGER_REGISTER_COUNT: usize = 64;
const STRING_REGISTER_COUNT: usize = 64;
const LIST_REGISTER_COUNT: usize = 16;

#[derive(Debug)]
pub struct RegisterTable {
    booleans: SmallVec<[Register<bool>; BOOLEAN_REGISTER_COUNT]>,
    bytes: SmallVec<[Register<u8>; BYTE_REGISTER_COUNT]>,
    characters: SmallVec<[Register<char>; CHARACTER_REGISTER_COUNT]>,
    floats: SmallVec<[Register<f64>; FLOAT_REGISTER_COUNT]>,
    integers: SmallVec<[Register<i64>; INTEGER_REGISTER_COUNT]>,
    strings: SmallVec<[Register<DustString>; STRING_REGISTER_COUNT]>,
    lists: SmallVec<[Register<AbstractList>; LIST_REGISTER_COUNT]>,
}

impl RegisterTable {
    pub fn new() -> Self {
        Self {
            booleans: smallvec![Register::Empty; BOOLEAN_REGISTER_COUNT],
            bytes: smallvec![Register::Empty; BYTE_REGISTER_COUNT],
            characters: smallvec![Register::Empty; CHARACTER_REGISTER_COUNT],
            floats: smallvec![Register::Empty; FLOAT_REGISTER_COUNT],
            integers: smallvec![Register::Empty; INTEGER_REGISTER_COUNT],
            strings: smallvec![Register::Empty; STRING_REGISTER_COUNT],
            lists: smallvec![Register::Empty; LIST_REGISTER_COUNT],
        }
    }

    pub fn get_boolean(&self, index: u16) -> &Register<bool> {
        let index = index as usize;

        if cfg!(debug_assertions) {
            self.booleans.get(index).unwrap()
        } else {
            unsafe { self.booleans.get(index).unwrap_unchecked() }
        }
    }

    pub fn get_boolean_mut(&mut self, index: u16) -> &mut Register<bool> {
        let index = index as usize;

        if cfg!(debug_assertions) {
            self.booleans.get_mut(index).unwrap()
        } else {
            unsafe { self.booleans.get_mut(index).unwrap_unchecked() }
        }
    }

    pub fn set_boolean(&mut self, index: u16, value: bool) {
        trace!("Set R_BOOL_{index} to value {value}");

        let index = index as usize;

        self.booleans[index] = Register::Value(value);

        Self::handle_growth(&mut self.booleans);
    }

    pub fn get_byte(&self, index: u16) -> &Register<u8> {
        let index = index as usize;

        if cfg!(debug_assertions) {
            self.bytes.get(index).unwrap()
        } else {
            unsafe { self.bytes.get(index).unwrap_unchecked() }
        }
    }

    pub fn get_byte_mut(&mut self, index: u16) -> &mut Register<u8> {
        let index = index as usize;

        if cfg!(debug_assertions) {
            self.bytes.get_mut(index).unwrap()
        } else {
            unsafe { self.bytes.get_mut(index).unwrap_unchecked() }
        }
    }

    pub fn set_byte(&mut self, index: u16, value: u8) {
        trace!("Set R_BYTE_{index} to value {value}");

        let index = index as usize;

        self.bytes[index] = Register::Value(value);

        Self::handle_growth(&mut self.bytes);
    }

    pub fn get_character(&self, index: u16) -> &Register<char> {
        let index = index as usize;

        if cfg!(debug_assertions) {
            self.characters.get(index).unwrap()
        } else {
            unsafe { self.characters.get(index).unwrap_unchecked() }
        }
    }

    pub fn get_character_mut(&mut self, index: u16) -> &mut Register<char> {
        let index = index as usize;

        if cfg!(debug_assertions) {
            self.characters.get_mut(index).unwrap()
        } else {
            unsafe { self.characters.get_mut(index).unwrap_unchecked() }
        }
    }

    pub fn set_character(&mut self, index: u16, value: char) {
        trace!("Set R_CHAR_{index} to value {value}");

        let index = index as usize;

        self.characters[index] = Register::Value(value);

        Self::handle_growth(&mut self.characters);
    }

    pub fn get_float(&self, index: u16) -> &Register<f64> {
        let index = index as usize;

        if cfg!(debug_assertions) {
            self.floats.get(index).unwrap()
        } else {
            unsafe { self.floats.get(index).unwrap_unchecked() }
        }
    }

    pub fn get_float_mut(&mut self, index: u16) -> &mut Register<f64> {
        let index = index as usize;

        if cfg!(debug_assertions) {
            self.floats.get_mut(index).unwrap()
        } else {
            unsafe { self.floats.get_mut(index).unwrap_unchecked() }
        }
    }

    pub fn set_float(&mut self, index: u16, value: f64) {
        trace!("Set R_FLOAT_{index} to value {value}");

        let index = index as usize;

        self.floats[index] = Register::Value(value);

        Self::handle_growth(&mut self.floats);
    }

    pub fn get_integer(&self, index: u16) -> &Register<i64> {
        let index = index as usize;

        let register = if cfg!(debug_assertions) {
            self.integers.get(index).unwrap()
        } else {
            unsafe { self.integers.get(index).unwrap_unchecked() }
        };

        match register {
            Register::Value(_) => register,
            Register::Pointer(pointer) => unsafe { &**pointer },
            Register::Empty => panic!("Expected a non-empty register"),
        }
    }

    pub fn get_integer_mut(&mut self, index: u16) -> &mut Register<i64> {
        let index = index as usize;

        if cfg!(debug_assertions) {
            self.integers.get_mut(index).unwrap()
        } else {
            unsafe { self.integers.get_mut(index).unwrap_unchecked() }
        }
    }

    pub fn get_many_integer_mut<I, const N: usize>(
        &mut self,
        indices: [I; N],
    ) -> [&mut <I as std::slice::SliceIndex<[Register<i64>]>>::Output; N]
    where
        I: GetManyMutIndex + SliceIndex<[Register<i64>]>,
    {
        if cfg!(debug_assertions) {
            self.integers.get_many_mut(indices).unwrap()
        } else {
            unsafe { self.integers.get_many_mut(indices).unwrap_unchecked() }
        }
    }

    pub fn set_integer(&mut self, index: u16, value: i64) {
        trace!("Set R_INT_{index} to value {value}");

        let index = index as usize;

        self.integers[index] = Register::Value(value);

        Self::handle_growth(&mut self.integers);
    }

    pub fn get_string(&self, index: u16) -> &Register<DustString> {
        let index = index as usize;

        if cfg!(debug_assertions) {
            self.strings.get(index).unwrap()
        } else {
            unsafe { self.strings.get(index).unwrap_unchecked() }
        }
    }

    pub fn get_string_mut(&mut self, index: u16) -> &mut Register<DustString> {
        let index = index as usize;

        if cfg!(debug_assertions) {
            self.strings.get_mut(index).unwrap()
        } else {
            unsafe { self.strings.get_mut(index).unwrap_unchecked() }
        }
    }

    pub fn set_string(&mut self, index: u16, value: DustString) {
        trace!("Set R_STR_{index} to value {value}");

        let index = index as usize;

        self.strings[index] = Register::Value(value);

        Self::handle_growth(&mut self.strings);
    }

    fn handle_growth<T: Clone, const REGISTER_COUNT: usize>(
        registers: &mut SmallVec<[Register<T>; REGISTER_COUNT]>,
    ) {
        if REGISTER_COUNT >= registers.len() {
            let new_length = registers.len() + REGISTER_COUNT;

            registers.resize(new_length, Register::Empty);
        }
    }
}

impl Default for RegisterTable {
    fn default() -> Self {
        Self::new()
    }
}
