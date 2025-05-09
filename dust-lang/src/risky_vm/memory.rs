use crate::{AbstractList, DustString, value::AbstractFunction};

#[derive(Debug)]
pub struct Memory {
    pub booleans: Vec<Slot<bool>>,
    pub bytes: Vec<Slot<u8>>,
    pub characters: Vec<Slot<char>>,
    pub floats: Vec<Slot<f64>>,
    pub integers: Vec<Slot<i64>>,
    pub strings: Vec<Slot<DustString>>,
    pub lists: Vec<Slot<AbstractList>>,
    pub functions: Vec<Slot<AbstractFunction>>,

    pub register_table: RegisterTable,
}

#[derive(Debug, Default)]
pub struct RegisterTable {
    pub booleans: RegisterList<bool>,
    pub bytes: RegisterList<u8>,
    pub characters: RegisterList<char>,
    pub floats: RegisterList<f64>,
    pub integers: RegisterList<i64>,
    pub strings: RegisterList<DustString>,
    pub lists: RegisterList<AbstractList>,
    pub functions: RegisterList<AbstractFunction>,
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

impl<T> RegisterList<T> {
    pub fn get(&self, index: u16) -> &T {
        match index {
            0 => &self.r_0,
            1 => &self.r_1,
            2 => &self.r_2,
            3 => &self.r_3,
            4 => &self.r_4,
            5 => &self.r_5,
            6 => &self.r_6,
            7 => &self.r_7,
            8 => &self.r_8,
            9 => &self.r_9,
            invalid => panic!("Invalid register index: {invalid}"),
        }
    }

    pub fn get_owned(self, index: u16) -> T {
        match index {
            0 => self.r_0,
            1 => self.r_1,
            2 => self.r_2,
            3 => self.r_3,
            4 => self.r_4,
            5 => self.r_5,
            6 => self.r_6,
            7 => self.r_7,
            8 => self.r_8,
            9 => self.r_9,
            invalid => panic!("Invalid register index: {invalid}"),
        }
    }

    pub fn set(&mut self, index: u16, new_value: T) {
        match index {
            0 => self.r_0 = new_value,
            1 => self.r_1 = new_value,
            2 => self.r_2 = new_value,
            3 => self.r_3 = new_value,
            4 => self.r_4 = new_value,
            5 => self.r_5 = new_value,
            6 => self.r_6 = new_value,
            7 => self.r_7 = new_value,
            8 => self.r_8 = new_value,
            9 => self.r_9 = new_value,
            invalid => panic!("Invalid register index: {invalid}"),
        }
    }
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

#[derive(Clone, Copy, Debug)]
pub struct Slot<T> {
    value: T,
    is_closed: bool,
}

impl<T> Slot<T> {
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

impl<T: Copy> Slot<T> {
    pub fn copy_value(&self) -> T {
        self.value
    }
}

impl<T: Clone> Slot<T> {
    pub fn clone_value(&self) -> T {
        self.value.clone()
    }
}

impl<T: Default> Default for Slot<T> {
    fn default() -> Self {
        Self {
            value: Default::default(),
            is_closed: false,
        }
    }
}
