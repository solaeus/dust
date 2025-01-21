use core::slice::GetManyMutIndex;
use std::slice::SliceIndex;

use smallvec::{SmallVec, smallvec};
use tracing::trace;

use crate::DustString;

use super::Pointer;

#[derive(Debug, Clone)]
pub enum Register<T: Clone> {
    Empty,
    Value(T),
    Pointer(Pointer),
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

#[derive(Debug)]
pub struct RegisterTable {
    booleans: SmallVec<[Register<bool>; 64]>,
    bytes: SmallVec<[Register<u8>; 64]>,
    characters: SmallVec<[Register<char>; 64]>,
    floats: SmallVec<[Register<f64>; 64]>,
    integers: SmallVec<[Register<i64>; 64]>,
    strings: SmallVec<[Register<DustString>; 64]>,
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

    pub fn get_integer(&self, index: u16) -> &Register<i64> {
        let index = index as usize;

        let register = if cfg!(debug_assertions) {
            self.integers.get(index).unwrap()
        } else {
            unsafe { self.integers.get(index).unwrap_unchecked() }
        };

        match register {
            Register::Value(_) => register,
            Register::Pointer(pointer) => match pointer {
                Pointer::RegisterInteger(register_index) => self.get_integer(*register_index),
                _ => todo!(),
            },
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

    pub fn set_integer(&mut self, index: u16, value: i64) {
        trace!("Set R_INT_{index} to value {value}");

        let index = index as usize;

        self.integers[index] = Register::Value(value);
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
}

impl Default for RegisterTable {
    fn default() -> Self {
        Self::new()
    }
}
