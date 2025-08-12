use std::{
    sync::{Arc, RwLock},
    thread::{Builder as ThreadBuilder, JoinHandle},
};

use cranelift::prelude::{
    Type,
    types::{I32, I64},
};
use tracing::{Level, info, span, trace};

use crate::{
    Program, Value,
    instruction::OperandType,
    jit_vm::call_stack::{new_call_stack, sizes::CALL_FRAME_SIZE},
};

use super::{
    Cell, Register,
    jit_compiler::{JitCompiler, JitError},
};

pub struct Thread {
    pub handle: JoinHandle<Result<Option<Value>, JitError>>,
}

impl Thread {
    pub fn spawn(
        program: Program,
        _cells: Arc<RwLock<Vec<Cell>>>,
        _threads: Arc<RwLock<Vec<Thread>>>,
    ) -> Result<Self, JitError> {
        let name = program
            .main_chunk
            .name
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or_else(|| "anonymous".to_string());

        info!("Spawning thread {name}");

        let handle = ThreadBuilder::new()
            .name(name)
            .spawn(|| run(program))
            .expect("Failed to spawn thread");

        Ok(Thread { handle })
    }
}

fn run(program: Program) -> Result<Option<Value>, JitError> {
    let span = span!(Level::TRACE, "Thread");
    let _enter = span.enter();

    let mut jit = JitCompiler::new(&program);
    let jit_logic = jit.compile()?;
    let mut call_stack = new_call_stack(CALL_FRAME_SIZE * 10);
    let mut call_stack_len = 0;
    let mut register_stack = vec![Register { empty: () }; 1024];
    let mut return_register = Register { empty: () };
    let mut return_type = OperandType::NONE;

    trace!("JIT compiled successfully");

    loop {
        let thread_status = (jit_logic)(
            call_stack.as_mut_ptr(),
            &mut call_stack_len,
            register_stack.as_mut_ptr(),
            &mut return_register,
            &mut return_type,
        );

        match thread_status {
            ThreadStatus::Return => break,
            ThreadStatus::ResizeCallStack => todo!(),
            ThreadStatus::ResizeRegisterStack => todo!(),
            ThreadStatus::ErrorFunctionIndexOutOfBounds => todo!(),
        }
    }

    trace!("JIT execution completed with type {return_type}");

    match return_type {
        OperandType::NONE => Ok(None),
        OperandType::BOOLEAN => {
            let boolean = unsafe { return_register.boolean };

            Ok(Some(Value::Boolean(boolean)))
        }
        OperandType::BYTE => {
            let byte = unsafe { return_register.byte };

            Ok(Some(Value::Byte(byte)))
        }
        OperandType::INTEGER => {
            let integer = unsafe { return_register.integer };

            Ok(Some(Value::Integer(integer)))
        }
        _ => todo!(),
    }
}

#[repr(C)]
pub enum ThreadStatus {
    Return = 0,
    ResizeCallStack = 1,
    ResizeRegisterStack = 2,
    ErrorFunctionIndexOutOfBounds = 3,
}

impl ThreadStatus {
    pub const CRANELIFT_TYPE: Type = match size_of::<ThreadStatus>() {
        4 => I32,
        8 => I64,
        _ => panic!("Unsupported ThreadStatus size"),
    };
}
