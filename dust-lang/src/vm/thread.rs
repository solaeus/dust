use std::{
    sync::{Arc, RwLock},
    thread::{Builder as ThreadBuilder, JoinHandle},
};

use tracing::{Level, info, span};

use crate::{JitChunk, Value, vm::Register};

use super::{CallFrame, Cell, Object};

pub struct ThreadHandle {
    pub handle: JoinHandle<Option<Value>>,
}

impl ThreadHandle {
    pub fn new(
        main_chunk: JitChunk,
        chunks: Arc<Vec<JitChunk>>,
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
pub struct Thread {
    pub(crate) should_exit: bool,
    pub(crate) return_value: Option<Value>,

    main_chunk: JitChunk,
    chunks: Arc<Vec<JitChunk>>,

    call_stack: Vec<CallFrame>,

    object_pool: Vec<Object>,

    threads: Arc<RwLock<Vec<ThreadHandle>>>,
    cells: Arc<RwLock<Vec<Cell>>>,
}

impl Thread {
    fn run(mut self) -> Option<Value> {
        let span = span!(Level::INFO, "Thread");
        let _enter = span.enter();

        info!("Starting thread");

        let mut register_stack = vec![Register::default(); self.main_chunk.register_count];
        let register_count = self.main_chunk.register_count;
        let mut call = CallFrame {
            ip: 0,
            chunk: &self.main_chunk,
            is_end_of_stack: true,
            registers: register_stack.as_mut_ptr(),
            register_count,
            return_address: 0,
            return_value: None,
        };
        (self.main_chunk.logic)(&mut self, &mut call);

        self.return_value
    }
}
