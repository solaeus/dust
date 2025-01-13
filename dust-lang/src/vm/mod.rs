//! Virtual machine and errors
mod function_call;
mod run_action;
mod stack;
mod thread;

use std::{
    fmt::{self, Debug, Display, Formatter},
    sync::Arc,
    thread::Builder,
};

pub use function_call::FunctionCall;
pub use run_action::RunAction;
pub(crate) use run_action::get_next_action;
pub use stack::Stack;
pub use thread::{Thread, ThreadData};

use crossbeam_channel::bounded;
use tracing::{Level, span};

use crate::{Chunk, DustError, Value, compile};

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
        let mut main_thread = Thread::new(Arc::new(self.main_chunk));
        let (tx, rx) = bounded(1);
        let _ = Builder::new().name(thread_name).spawn(move || {
            let value_option = main_thread.run();
            let _ = tx.send(value_option);
        });

        rx.recv().unwrap_or(None)
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
    Register(u16),
    Constant(u16),
    Stack(usize, u16),
}

impl Display for Pointer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Register(index) => write!(f, "PR{}", index),
            Self::Constant(index) => write!(f, "PC{}", index),
            Self::Stack(call_index, register_index) => {
                write!(f, "PS{}R{}", call_index, register_index)
            }
        }
    }
}
