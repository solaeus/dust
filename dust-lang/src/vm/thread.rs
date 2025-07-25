use std::{
    sync::{Arc, RwLock},
    thread::{Builder as ThreadBuilder, JoinHandle},
};

use tracing::{Level, info, span};

use crate::{Address, JitChunk, StrippedChunk, Value, instruction::OperandType, vm::Register};

use super::{CallFrame, Cell, Object};

pub struct ThreadHandle {
    pub handle: JoinHandle<Option<Value<StrippedChunk>>>,
}

impl ThreadHandle {
    pub fn new(
        chunks: Arc<Vec<Arc<JitChunk>>>,
        cells: Arc<RwLock<Vec<Cell>>>,
        threads: Arc<RwLock<Vec<ThreadHandle>>>,
    ) -> Self {
        let handle = ThreadBuilder::new()
            .spawn(move || {
                let runner = Thread {
                    object_pool: Vec::new(),
                    call_stack: Vec::new(),
                    threads,
                    cells,
                    chunks,
                    return_value: None,
                    should_exit: false,
                };

                runner.run()
            })
            .expect("Failed to spawn thread");

        ThreadHandle { handle }
    }
}

#[repr(C)]
pub struct Thread<'a> {
    pub(crate) should_exit: bool,
    pub(crate) return_value: Option<Value>,

    chunks: Arc<Vec<Arc<JitChunk>>>,

    call_stack: Vec<CallFrame<'a>>,

    object_pool: Vec<Object>,

    threads: Arc<RwLock<Vec<ThreadHandle>>>,
    cells: Arc<RwLock<Vec<Cell>>>,
}

impl<'a> Thread<'a> {
    fn run(mut self) -> Option<Value<StrippedChunk>> {
        let span = span!(Level::INFO, "Thread");
        let _enter = span.enter();

        info!("Starting thread");

        let main_chunk = &self.chunks[0];
        let mut register_stack = vec![Register::default(); main_chunk.register_count];
        let register_count = main_chunk.register_count;
        let mut call = CallFrame::new(
            main_chunk.clone(),
            &mut register_stack[0..register_count],
            true,
            Address::default(),
            OperandType::NONE,
        );

        (main_chunk.logic)(&mut self, &mut call);

        self.return_value
    }
}
