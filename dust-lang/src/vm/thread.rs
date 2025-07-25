use std::{
    sync::{Arc, RwLock},
    thread::{Builder as ThreadBuilder, JoinHandle},
};

use tracing::{Level, info, span};

use crate::{Address, JitChunk, Value, instruction::OperandType, vm::Register};

use super::{CallFrame, Cell, Object};

pub struct ThreadHandle {
    pub handle: JoinHandle<Option<Value>>,
}

impl ThreadHandle {
    pub fn new(
        main_chunk: Arc<JitChunk>,
        chunks: Arc<Vec<Arc<JitChunk>>>,
        cells: Arc<RwLock<Vec<Cell>>>,
        threads: Arc<RwLock<Vec<ThreadHandle>>>,
    ) -> Self {
        let handle = ThreadBuilder::new()
            .spawn(move || {
                let runner = Thread {
                    main_chunk,
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

    main_chunk: Arc<JitChunk>,
    chunks: Arc<Vec<Arc<JitChunk>>>,

    call_stack: Vec<CallFrame<'a>>,

    object_pool: Vec<Object>,

    threads: Arc<RwLock<Vec<ThreadHandle>>>,
    cells: Arc<RwLock<Vec<Cell>>>,
}

impl<'a> Thread<'a> {
    fn run(mut self) -> Option<Value> {
        let span = span!(Level::INFO, "Thread");
        let _enter = span.enter();

        info!("Starting thread");

        let mut register_stack = vec![Register::default(); self.main_chunk.register_count];
        let register_count = self.main_chunk.register_count;
        let mut call = CallFrame::new(
            self.main_chunk.clone(),
            &mut register_stack[0..register_count],
            true,
            Address::default(),
            OperandType::NONE,
        );

        (self.main_chunk.logic)(&mut self, &mut call);

        self.return_value
    }
}
