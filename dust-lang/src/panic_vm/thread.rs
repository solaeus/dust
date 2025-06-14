use std::{
    mem::replace,
    sync::{Arc, RwLock},
    thread::{Builder, JoinHandle},
};

use tracing::{Level, info, span, warn};

use crate::{
    AbstractList, Address, Chunk, ConcreteValue, DustString, Operation,
    instruction::{
        Add, Call, CallNative, Close, Divide, Equal, Jump, Less, LessEqual, Modulo, Multiply,
        Negate, Return, Subtract, Test,
    },
};

use super::{CallFrame, Memory, macros::*};

pub struct Thread {
    pub handle: JoinHandle<Option<ConcreteValue>>,
}

impl Thread {
    pub fn new(chunk: Arc<Chunk>, threads: Arc<RwLock<Vec<Thread>>>) -> Self {
        let mut runner = ThreadRunner {
            chunk: Arc::clone(&chunk),
            threads,
        };

        let handle = Builder::new()
            .name(
                chunk
                    .name
                    .as_ref()
                    .map(|name| name.to_string())
                    .unwrap_or_else(|| "anonymous".to_string()),
            )
            .spawn(move || runner.run())
            .expect("Failed to spawn thread");

        Thread { handle }
    }
}

#[derive(Clone)]
struct ThreadRunner {
    chunk: Arc<Chunk>,
    threads: Arc<RwLock<Vec<Thread>>>,
}

impl ThreadRunner {
    fn run(&mut self) -> Option<ConcreteValue> {
        let span = span!(Level::INFO, "Thread");
        let _enter = span.enter();

        info!(
            "Starting thread {}",
            self.chunk
                .name
                .as_ref()
                .map(|name| name.as_str())
                .unwrap_or_default()
        );

        let mut call_stack = Vec::<CallFrame>::new();
        let mut memory_stack = Vec::<Memory>::new();

        let mut call = CallFrame::new(Arc::clone(&self.chunk), Address::default());
        let mut memory = Memory::new(&call.chunk);

        loop {
            let instructions = &call.chunk.instructions;
            let ip = call.ip;
            call.ip += 1;

            assert!(ip < instructions.len(), "IP out of bounds");

            let instruction = instructions[ip];
            let operation = instruction.operation();

            info!("IP = {ip} Run {operation}");

            match operation {
                Operation::NO_OP => {
                    warn!("Running NO_OP instruction");
                }
                _ => todo!("Handle operation: {operation}"),
            }
        }
    }
}
