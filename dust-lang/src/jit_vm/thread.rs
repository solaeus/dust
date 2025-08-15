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
    List, Program, Value,
    instruction::OperandType,
    jit_vm::{ObjectPool, call_stack::new_call_stack, object::ObjectValue},
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

    let mut call_stack_used_length = 0;
    let mut call_stack_allocated_length = 256;
    let mut call_stack = new_call_stack(call_stack_allocated_length);

    let mut register_stack_used_length = 0;
    let mut register_stack_allocated_length = 256;
    let mut register_stack = vec![Register { empty: () }; register_stack_allocated_length];

    let mut object_pool = ObjectPool::new();

    let mut return_register = Register { empty: () };
    let mut return_type = OperandType::NONE;

    let mut thread_context = ThreadContext {
        call_stack_vec_pointer: &mut call_stack,
        call_stack_buffer_pointer: call_stack.as_mut_ptr(),
        call_stack_allocated_length_pointer: &mut call_stack_allocated_length,
        call_stack_used_length_pointer: &mut call_stack_used_length,
        register_stack_vec_pointer: &mut register_stack,
        register_stack_buffer_pointer: register_stack.as_mut_ptr(),
        register_stack_allocated_length_pointer: &mut register_stack_allocated_length,
        register_stack_used_length_pointer: &mut register_stack_used_length,
        object_pool_pointer: &mut object_pool,
        return_register_pointer: &mut return_register,
        return_type_pointer: &mut return_type,
    };

    trace!("JIT compiled successfully");

    loop {
        let thread_status = (jit_logic)(&mut thread_context);

        match thread_status {
            ThreadResult::Return => break,
            ThreadResult::ErrorFunctionIndexOutOfBounds => todo!(),
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
        OperandType::CHARACTER => {
            let character = unsafe { return_register.character };

            Ok(Some(Value::Character(character)))
        }
        OperandType::FLOAT => {
            let float = unsafe { return_register.float };

            Ok(Some(Value::Float(float)))
        }
        OperandType::INTEGER => {
            let integer = unsafe { return_register.integer };

            Ok(Some(Value::Integer(integer)))
        }
        OperandType::STRING => {
            let object_pointer = unsafe { return_register.object_pointer };
            let object = unsafe { &*object_pointer };
            let string = object
                .as_string()
                .cloned()
                .ok_or(JitError::InvalidConstantType {
                    expected_type: OperandType::STRING,
                })?;

            Ok(Some(Value::String(string)))
        }
        OperandType::LIST_BOOLEAN => {
            let object_pointer = unsafe { return_register.object_pointer };
            let object = unsafe { &*object_pointer };
            let booleans = if let ObjectValue::BooleanList(booleans) = &object.value {
                booleans.clone()
            } else {
                return Err(JitError::InvalidConstantType {
                    expected_type: OperandType::LIST_BOOLEAN,
                });
            };

            Ok(Some(Value::List(List::Boolean(booleans))))
        }
        OperandType::LIST_BYTE => {
            let object_pointer = unsafe { return_register.object_pointer };
            let object = unsafe { &*object_pointer };
            let bytes = if let ObjectValue::ByteList(bytes) = &object.value {
                bytes.clone()
            } else {
                return Err(JitError::InvalidConstantType {
                    expected_type: OperandType::LIST_BYTE,
                });
            };

            Ok(Some(Value::List(List::Byte(bytes))))
        }
        OperandType::LIST_CHARACTER => {
            let object_pointer = unsafe { return_register.object_pointer };
            let object = unsafe { &*object_pointer };
            let characters = if let ObjectValue::CharacterList(characters) = &object.value {
                characters.clone()
            } else {
                return Err(JitError::InvalidConstantType {
                    expected_type: OperandType::LIST_CHARACTER,
                });
            };

            Ok(Some(Value::List(List::Character(characters))))
        }
        OperandType::LIST_FLOAT => {
            let object_pointer = unsafe { return_register.object_pointer };
            let object = unsafe { &*object_pointer };
            let floats = if let ObjectValue::FloatList(floats) = &object.value {
                floats.clone()
            } else {
                return Err(JitError::InvalidConstantType {
                    expected_type: OperandType::LIST_FLOAT,
                });
            };

            Ok(Some(Value::List(List::Float(floats))))
        }
        OperandType::LIST_INTEGER => {
            let object_pointer = unsafe { return_register.object_pointer };
            let object = unsafe { &*object_pointer };
            let integers = if let ObjectValue::IntegerList(integers) = &object.value {
                integers.clone()
            } else {
                return Err(JitError::InvalidConstantType {
                    expected_type: OperandType::LIST_INTEGER,
                });
            };

            Ok(Some(Value::List(List::Integer(integers))))
        }
        OperandType::LIST_STRING => {
            let object_pointer = unsafe { return_register.object_pointer };
            let object = unsafe { &*object_pointer };
            let strings = if let ObjectValue::ObjectList(objects) = &object.value {
                objects
                    .iter()
                    .map(|object_pointer| {
                        let object = unsafe { &**object_pointer };

                        object
                            .as_string()
                            .cloned()
                            .ok_or(JitError::InvalidConstantType {
                                expected_type: OperandType::STRING,
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                return Err(JitError::InvalidConstantType {
                    expected_type: OperandType::LIST_STRING,
                });
            };

            Ok(Some(Value::List(List::String(strings))))
        }
        _ => todo!(),
    }
}

#[repr(C)]
pub enum ThreadResult {
    Return = 0,
    ErrorFunctionIndexOutOfBounds = 3,
}

impl ThreadResult {
    pub const CRANELIFT_TYPE: Type = match size_of::<ThreadResult>() {
        4 => I32,
        8 => I64,
        _ => panic!("Unsupported ThreadStatus size"),
    };
}

#[repr(C)]
pub struct ThreadContext {
    pub call_stack_vec_pointer: *mut Vec<u8>,
    pub call_stack_buffer_pointer: *mut u8,
    pub call_stack_allocated_length_pointer: *mut usize,
    pub call_stack_used_length_pointer: *mut usize,

    pub register_stack_vec_pointer: *mut Vec<Register>,
    pub register_stack_buffer_pointer: *mut Register,
    pub register_stack_allocated_length_pointer: *mut usize,
    pub register_stack_used_length_pointer: *mut usize,

    pub object_pool_pointer: *mut ObjectPool,

    pub return_register_pointer: *mut Register,
    pub return_type_pointer: *mut OperandType,
}
