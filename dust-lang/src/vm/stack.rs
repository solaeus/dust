use std::{
    fmt::{self, Debug, Display, Formatter},
    ops::{Index, IndexMut, Range},
};

use crate::DustString;

use super::VmError;

#[derive(Clone, PartialEq)]
pub struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack {
            items: Vec::with_capacity(1),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Stack {
            items: Vec::with_capacity(capacity),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get_unchecked(&self, index: usize) -> &T {
        if cfg!(debug_assertions) {
            assert!(index < self.len(), "{}", VmError::StackUnderflow);

            &self.items[index]
        } else {
            unsafe { self.items.get_unchecked(index) }
        }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    pub fn last(&self) -> Option<&T> {
        self.items.last()
    }

    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.items.last_mut()
    }

    pub fn pop_unchecked(&mut self) -> T {
        if cfg!(debug_assertions) {
            assert!(!self.is_empty(), "{}", VmError::StackUnderflow);

            self.items.pop().unwrap()
        } else {
            unsafe { self.items.pop().unwrap_unchecked() }
        }
    }

    pub fn last_unchecked(&self) -> &T {
        if cfg!(debug_assertions) {
            assert!(!self.is_empty(), "{}", VmError::StackUnderflow);

            self.items.last().unwrap()
        } else {
            unsafe { self.items.last().unwrap_unchecked() }
        }
    }

    pub fn last_mut_unchecked(&mut self) -> &mut T {
        if cfg!(debug_assertions) {
            assert!(!self.is_empty(), "{}", VmError::StackUnderflow);

            self.items.last_mut().unwrap()
        } else {
            unsafe { self.items.last_mut().unwrap_unchecked() }
        }
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<Range<usize>> for Stack<T> {
    type Output = [T];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.items[index]
    }
}

impl<T> IndexMut<Range<usize>> for Stack<T> {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        &mut self.items[index]
    }
}

impl<T: Debug> Debug for Stack<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self.items)
    }
}

impl Display for Stack<FunctionCall> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "-- DUST CALL STACK --")?;

        for function_call in self.items.iter().rev() {
            writeln!(f, "{function_call}")?;
        }

        writeln!(f, "--")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionCall {
    pub name: Option<DustString>,
    pub return_register: u8,
    pub ip: usize,
}

impl Display for FunctionCall {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let FunctionCall {
            name,
            return_register,
            ..
        } = self;
        let name = name
            .as_ref()
            .map(|name| name.as_str())
            .unwrap_or("anonymous");

        write!(f, "{name} (Return register: {return_register})")
    }
}
