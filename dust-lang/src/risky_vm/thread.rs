use std::{sync::Arc, thread::JoinHandle};

use tracing::info;

use crate::{
    Chunk, DustString, Span, Value,
    risky_vm::{CallFrame, runner::RUNNERS},
};

use super::{HeapSlotTable, Memory, RegisterTable};

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
            register_table: RegisterTable::default(),
            heap_slot_table: HeapSlotTable::new(chunk.as_ref()),
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
        let current_frame = self.current_frame();

        current_frame.chunk.positions[current_frame.ip]
    }

    pub fn current_frame(&self) -> &CallFrame {
        if cfg!(debug_assertions) {
            self.call_stack.last().unwrap()
        } else {
            unsafe { self.call_stack.last().unwrap_unchecked() }
        }
    }

    pub fn current_frame_mut(&mut self) -> &mut CallFrame {
        if cfg!(debug_assertions) {
            self.call_stack.last_mut().unwrap()
        } else {
            unsafe { self.call_stack.last_mut().unwrap_unchecked() }
        }
    }

    pub fn current_memory(&self) -> &Memory {
        if cfg!(debug_assertions) {
            &self.memory_stack.last().unwrap()
        } else {
            unsafe { &self.memory_stack.last().unwrap_unchecked() }
        }
    }

    pub fn current_memory_mut(&mut self) -> &mut Memory {
        if cfg!(debug_assertions) {
            self.memory_stack.last_mut().unwrap()
        } else {
            unsafe { self.memory_stack.last_mut().unwrap_unchecked() }
        }
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
