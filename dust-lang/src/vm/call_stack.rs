use std::fmt::{self, Debug, Display, Formatter};

use super::{FunctionCall, VmError};

#[derive(Clone, PartialEq)]
pub struct CallStack {
    pub calls: Vec<FunctionCall>,
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

    pub fn push(&mut self, call: FunctionCall) {
        self.calls.push(call);
    }

    pub fn pop(&mut self) -> Option<FunctionCall> {
        self.calls.pop()
    }

    pub fn last(&self) -> Option<&FunctionCall> {
        self.calls.last()
    }

    pub fn pop_or_panic(&mut self) -> FunctionCall {
        assert!(!self.is_empty(), "{}", VmError::CallStackUnderflow);

        self.calls.pop().unwrap()
    }

    pub fn last_or_panic(&self) -> &FunctionCall {
        assert!(!self.is_empty(), "{}", VmError::CallStackUnderflow);

        self.calls.last().unwrap()
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

        for function_call in &self.calls {
            writeln!(f, "{function_call:?}")?;
        }

        Ok(())
    }
}
