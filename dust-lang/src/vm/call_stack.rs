use std::fmt::{self, Debug, Display, Formatter};

use super::FunctionCall;

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
}

impl Debug for CallStack {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for CallStack {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "-- DUST CALL STACK --")?;

        for FunctionCall { function, .. } in &self.calls {
            writeln!(f, "{function}")?;
        }

        Ok(())
    }
}
