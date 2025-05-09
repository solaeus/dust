use std::{sync::Arc, thread::JoinHandle};

use tracing::info;

use crate::{Chunk, DustString, Span, Value, risky_vm::runner::RUNNERS};

use super::{CallFrame, Memory, RegisterTable};

pub struct Thread {
    pub chunk: Arc<Chunk>,

    pub call_stack: Vec<CallFrame>,
    pub memory_stack: Vec<Memory>,

    pub current_call: CallFrame,
    pub current_memory: Memory,

    pub _spawned_threads: Vec<JoinHandle<()>>,
}

impl Thread {
    pub fn new(chunk: Arc<Chunk>) -> Self {
        let call_stack = Vec::with_capacity(chunk.prototypes.len() + 1);
        let main_call = CallFrame::new(Arc::clone(&chunk), 0);
        let memory_stack = Vec::with_capacity(chunk.prototypes.len() + 1);
        let main_memory = Memory {
            booleans: Vec::new(),
            bytes: Vec::new(),
            characters: Vec::new(),
            floats: Vec::new(),
            integers: Vec::new(),
            strings: Vec::new(),
            lists: Vec::new(),
            functions: Vec::new(),
            register_table: RegisterTable::default(),
        };

        Thread {
            chunk,
            call_stack,
            memory_stack,
            current_call: main_call,
            current_memory: main_memory,
            _spawned_threads: Vec::new(),
        }
    }

    fn current_position(&self) -> Span {
        self.current_call.chunk.positions[self.current_call.ip]
    }

    pub fn run(mut self) -> Option<Value> {
        info!(
            "Starting thread {}",
            self.chunk
                .name
                .clone()
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        loop {
            let instructions = &self.current_call.chunk.instructions;
            let ip = self.current_call.ip;
            self.current_call.ip += 1;

            assert!(ip < instructions.len(), "IP out of bounds");

            let instruction = instructions[ip];
            let operation = instruction.operation();

            info!("IP = {ip} Run {operation}");

            let runner = RUNNERS[operation.0 as usize];

            self = runner(instruction, self);
        }
    }
}
