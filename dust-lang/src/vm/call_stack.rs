use std::fmt::{self, Debug, Display, Formatter};

use crate::DustString;

use super::VmError;

#[derive(Clone, PartialEq)]
pub struct CallStack {
    calls: Vec<FunctionCall>,
}

impl CallStack {
    pub fn new() -> Self {
        CallStack {
            calls: Vec::with_capacity(1),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        CallStack {
            calls: Vec::with_capacity(capacity),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.calls.is_empty()
    }

    pub fn len(&self) -> usize {
        self.calls.len()
    }

    pub fn push(&mut self, call: FunctionCall) {
        self.calls.push(call);
    }

    pub fn pop(&mut self) -> Option<FunctionCall> {
        self.calls.pop()
    }

    pub fn last(&self) -> Option<&FunctionCall> {
        self.calls.last()
    }

    pub fn last_mut(&mut self) -> Option<&mut FunctionCall> {
        self.calls.last_mut()
    }

    pub fn pop_or_panic(&mut self) -> FunctionCall {
        assert!(!self.is_empty(), "{}", VmError::CallStackUnderflow);

        self.calls.pop().unwrap()
    }

    pub fn last_or_panic(&self) -> &FunctionCall {
        assert!(!self.is_empty(), "{}", VmError::CallStackUnderflow);

        self.calls.last().unwrap()
    }

    pub fn last_mut_or_panic(&mut self) -> &mut FunctionCall {
        assert!(!self.is_empty(), "{}", VmError::CallStackUnderflow);

        self.calls.last_mut().unwrap()
    }
}

impl Default for CallStack {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for CallStack {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for CallStack {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "-- DUST CALL STACK --")?;

        for function_call in self.calls.iter().rev() {
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
