use std::{
    sync::{Arc, RwLock},
    thread::{Builder as ThreadBuilder, JoinHandle},
};

use tracing::{Level, info, span};

use crate::{
    Address, Chunk, Value,
    instruction::OperandType,
    jit::{Jit, JitError},
    vm::Register,
};

use super::{CallFrame, Cell, ObjectPool};

pub struct Thread {
    pub handle: JoinHandle<Result<Option<Value>, JitError>>,
}

impl Thread {
    pub fn new(
        chunk: Chunk,
        cells: Arc<RwLock<Vec<Cell>>>,
        threads: Arc<RwLock<Vec<Thread>>>,
    ) -> Self {
        let name = chunk
            .name
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or_else(|| "anonymous".to_string());
        let handle = ThreadBuilder::new()
            .name(name)
            .spawn(move || {
                let runner = ThreadRunner {
                    object_pool: ObjectPool::new(),
                    call_stack: Vec::new(),
                    threads,
                    cells,
                    main_chunk: Arc::new(chunk),
                    return_value: None,
                    should_exit: false,
                };

                runner.run()
            })
            .expect("Failed to spawn thread");

        Thread { handle }
    }
}

#[repr(C)]
pub struct ThreadRunner<'a> {
    pub(crate) should_exit: bool,
    pub(crate) return_value: Option<Value>,
    pub(crate) object_pool: ObjectPool,

    main_chunk: Arc<Chunk>,

    call_stack: Vec<CallFrame<'a>>,

    threads: Arc<RwLock<Vec<Thread>>>,
    cells: Arc<RwLock<Vec<Cell>>>,
}

impl<'a> ThreadRunner<'a> {
    fn run(mut self) -> Result<Option<Value>, JitError> {
        let span = span!(Level::INFO, "Thread");
        let _enter = span.enter();

        info!(
            "Starting thread {}",
            self.main_chunk
                .name
                .as_ref()
                .map(|name| name.as_ref())
                .unwrap_or_default()
        );

        let mut jit = Jit::new(&self.main_chunk, &mut self.object_pool);
        let decoded_chunk = jit.compile()?;
        let register_count = self.main_chunk.register_tags.len();
        let mut register_stack = vec![Register { empty: () }; register_count];
        let mut call = CallFrame::new(
            Arc::clone(&self.main_chunk),
            &mut register_stack[0..register_count],
            true,
            Address::default(),
            OperandType::NONE,
        );

        (decoded_chunk.logic)(&mut self, &mut call);

        Ok(self.return_value)
    }
}
