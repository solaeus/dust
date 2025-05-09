use std::{sync::Arc, thread::JoinHandle};

use tracing::info;

use crate::{
    Address, Chunk, ConcreteValue, DustString, Value, instruction::AddressKind,
    risky_vm::runners::RUNNERS,
};

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

    pub fn resolve_boolean(&self, address: &Address) -> bool {
        match address.kind {
            AddressKind::BOOLEAN_MEMORY => *self
                .current_memory
                .booleans
                .get(address.index as usize)
                .unwrap()
                .as_value(),
            AddressKind::BOOLEAN_REGISTER => *self
                .current_memory
                .register_table
                .booleans
                .get(address.index),
            _ => unreachable!(),
        }
    }

    pub fn resolve_byte(&self, address: &Address) -> u8 {
        match address.kind {
            AddressKind::BOOLEAN_MEMORY => *self
                .current_memory
                .bytes
                .get(address.index as usize)
                .unwrap()
                .as_value(),
            AddressKind::BOOLEAN_REGISTER => {
                *self.current_memory.register_table.bytes.get(address.index)
            }
            _ => unreachable!(),
        }
    }

    pub fn resolve_string(&self, address: &Address) -> &DustString {
        match address.kind {
            AddressKind::BOOLEAN_MEMORY => self
                .current_memory
                .strings
                .get(address.index as usize)
                .unwrap()
                .as_value(),
            AddressKind::BOOLEAN_REGISTER => self
                .current_memory
                .register_table
                .strings
                .get(address.index),
            _ => unreachable!(),
        }
    }

    pub fn run(mut self) -> Option<ConcreteValue> {
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
            let runner = RUNNERS[operation.0 as usize];

            info!("IP = {ip} Run {operation}");

            self = runner(instruction, self);
        }
    }
}
