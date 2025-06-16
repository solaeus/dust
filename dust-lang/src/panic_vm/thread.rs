use std::{
    mem::replace,
    sync::{Arc, RwLock},
    thread::{Builder, JoinHandle},
};

use tracing::{Level, info, span, warn};

use crate::{
    Address, Chunk, DustString, FullChunk, Operation, Value,
    instruction::{
        Add, Call, CallNative, Close, Divide, Equal, Jump, Less, LessEqual, Load, MemoryKind,
        Modulo, Multiply, Negate, OperandType, Return, Subtract, Test,
    },
};

use super::{CallFrame, Cell, CellValue, Memory, macros::*};

pub struct Thread<C> {
    pub handle: JoinHandle<Option<Value<C>>>,
}

impl<C: 'static + Chunk + Send + Sync> Thread<C> {
    pub fn new(
        chunk: Arc<C>,
        cells: Arc<RwLock<Vec<Cell<C>>>>,
        threads: Arc<RwLock<Vec<Thread<C>>>>,
    ) -> Self {
        let name = chunk
            .name()
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or_else(|| "anonymous".to_string());
        let runner = ThreadRunner {
            chunk,
            threads,
            cells,
        };
        let handle = Builder::new()
            .name(name)
            .spawn(|| runner.run())
            .expect("Failed to spawn thread");

        Thread { handle }
    }
}

#[derive(Clone)]
struct ThreadRunner<C> {
    chunk: Arc<C>,
    threads: Arc<RwLock<Vec<Thread<C>>>>,
    cells: Arc<RwLock<Vec<Cell<C>>>>,
}

impl<C: Chunk> ThreadRunner<C> {
    fn run(mut self) -> Option<Value<C>> {
        let span = span!(Level::INFO, "Thread");
        let _enter = span.enter();

        info!(
            "Starting thread {}",
            self.chunk
                .name()
                .as_ref()
                .map(|name| name.as_str())
                .unwrap_or_default()
        );

        let mut call_stack = Vec::<CallFrame<C>>::new();
        let mut memory_stack = Vec::<Memory<C>>::new();

        let mut call = CallFrame::new(Arc::clone(&self.chunk), Address::default());
        let mut memory = Memory::new(&*call.chunk);

        loop {
            let instructions = &call.chunk.instructions();
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
                Operation::LOAD => {
                    // let Load {
                    //     destination,
                    //     operand,
                    //     r#type,
                    //     jump_next,
                    // } = Load::from(&instruction);

                    // match r#type {
                    //     OperandType::BOOLEAN => {
                    //         let boolean = get_boolean!(operand, memory, call.chunk, self.cells);

                    //         set_boolean!(destination, memory, self.cells, boolean);
                    //     }
                    //     OperandType::BYTE => {
                    //         let byte = get_byte!(operand, memory, call.chunk, self.cells);

                    //         set_byte!(destination, memory, self.cells, byte);
                    //     }
                    //     OperandType::CHARACTER => {
                    //         let character = get_character!(operand, memory, call.chunk, self.cells);

                    //         set_character!(destination, memory, self.cells, character);
                    //     }
                    //     OperandType::FLOAT => {
                    //         let float = get_float!(operand, memory, call.chunk, self.cells);

                    //         set_float!(destination, memory, self.cells, float);
                    //     }
                    //     OperandType::INTEGER => {
                    //         let integer = get_integer!(operand, memory, call.chunk, self.cells);

                    //         set_integer!(destination, memory, self.cells, integer);
                    //     }
                    //     OperandType::STRING => {
                    //         let string = get_string!(operand, memory, call.chunk, self.cells);

                    //         set_string!(destination, memory, self.cells, string);
                    //     }
                    //     OperandType::LIST => {
                    //         let list = get_list!(operand, memory, call.chunk, self.cells);

                    //         set_list!(destination, memory, self.cells, list);
                    //     }
                    //     OperandType::FUNCTION => {
                    //         let function = get_function!(operand, memory, &call.chunk, self.cells);

                    //         set_function!(destination, memory, self.cells, function);
                    //     }
                    //     _ => unreachable!(),
                    // }

                    // if jump_next {
                    //     call.ip += 1;
                    // }
                }
                _ => todo!("Handle operation: {operation}"),
            }
        }
    }
}
