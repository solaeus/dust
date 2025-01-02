//! Virtual machine and errors
mod call_stack;
mod error;
mod record;
mod run_action;
mod thread;

use std::{
    fmt::{self, Debug, Display, Formatter},
    sync::mpsc,
    thread::spawn,
};

pub use call_stack::{CallStack, FunctionCall};
pub use error::VmError;
pub use record::Record;
pub use run_action::RunAction;
pub use thread::{Thread, ThreadSignal};
use tracing::{span, Level};

use crate::{compile, Chunk, DustError, Value};

pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let chunk = compile(source)?;
    let vm = Vm::new(chunk);

    Ok(vm.run())
}

pub struct Vm {
    threads: Vec<Thread>,
}

impl Vm {
    pub fn new(chunk: Chunk) -> Self {
        let threads = vec![Thread::new(chunk)];

        debug_assert_eq!(1, threads.capacity());

        Self { threads }
    }

    pub fn run(mut self) -> Option<Value> {
        let span = span!(Level::INFO, "Run");
        let _enter = span.enter();

        if self.threads.len() == 1 {
            return self.threads[0].run();
        }

        let (tx, rx) = mpsc::channel();

        for mut thread in self.threads {
            let tx = tx.clone();

            spawn(move || {
                let return_value = thread.run();

                if let Some(value) = return_value {
                    tx.send(value).unwrap();
                }
            });
        }

        rx.into_iter().last()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Register {
    Empty,
    Value(Value),
    Pointer(Pointer),
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty"),
            Self::Value(value) => write!(f, "{}", value),
            Self::Pointer(pointer) => write!(f, "{}", pointer),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Pointer {
    Stack(u8),
    Constant(u8),
}

impl Display for Pointer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Stack(index) => write!(f, "R{}", index),
            Self::Constant(index) => write!(f, "C{}", index),
        }
    }
}
