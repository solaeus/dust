use std::{
    sync::Arc,
    thread::{Builder as ThreadBuilder, JoinHandle},
};

use bumpalo::Bump;
use cranelift::prelude::{
    Type as CraneliftType,
    types::{I32, I64},
};
use tracing::{Level, debug, info, span};

use crate::{
    List, Type, Value,
    dust_crate::Program,
    instruction::OperandType,
    jit_vm::{ObjectPool, RegisterTag, call_stack::new_call_stack, object::ObjectValue},
    resolver::{FunctionTypeNode, TypeNode},
};

use super::{
    Register,
    jit_compiler::{JitCompiler, JitError},
};

pub struct Thread {
    pub handle: JoinHandle<Result<Option<Value>, JitError>>,
}

impl Thread {
    pub fn spawn(
        thread_name: String,
        program: Arc<Program>,
        prototype_index: u16,
        minimum_object_heap: usize,
        minimum_object_sweep: usize,
    ) -> Result<Self, JitError> {
        info!("Spawning thread for proto_{prototype_index}");

        let handle = ThreadBuilder::new()
            .name(thread_name)
            .spawn(move || run(program, minimum_object_sweep, minimum_object_heap))
            .expect("Failed to spawn thread");

        Ok(Thread { handle })
    }
}

fn run(
    program: Arc<Program>,
    minimum_object_heap: usize,
    minimum_object_sweep: usize,
) -> Result<Option<Value>, JitError> {
    let span = span!(Level::TRACE, "Thread");
    let _enter = span.enter();

    let mut jit = JitCompiler::new(&program);
    let jit_logic = jit.compile()?;

    info!("JIT compilation complete");

    let mut call_stack_used_length = 0;
    let mut call_stack_allocated_length = if program.prototypes.len() == 1 { 1 } else { 64 };
    let mut call_stack = new_call_stack(call_stack_allocated_length);

    let mut register_stack_used_length = 0;
    let mut register_stack_allocated_length = if program.prototypes.len() == 1 {
        program.main_chunk().register_count as usize
    } else {
        1024 * 1024 * 4
    };
    let mut register_stack = vec![Register { empty: () }; register_stack_allocated_length];
    let mut register_tags = vec![RegisterTag::EMPTY; register_stack_allocated_length];
    let bump_arena = Bump::with_capacity(minimum_object_heap);

    let mut object_pool = ObjectPool::new(&bump_arena, minimum_object_sweep, minimum_object_heap);

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

        register_tags_vec_pointer: &mut register_tags,
        register_tags_buffer_pointer: register_tags.as_mut_ptr(),

        object_pool_pointer: &mut object_pool,

        return_register_pointer: &mut return_register,
        return_type_pointer: &mut return_type,
    };

    loop {
        let thread_status = (jit_logic)(&mut thread_context);

        match thread_status {
            ThreadResult::Return => break,
            ThreadResult::ErrorFunctionIndexOutOfBounds => todo!(),
        }
    }

    info!("JIT execution completed with type {return_type}");
    debug!("{}", object_pool.report());

    match return_type {
        OperandType::NONE => Ok(None),
        OperandType::BOOLEAN => {
            let boolean = get_boolean_from_register(return_register);

            Ok(Some(Value::Boolean(boolean)))
        }
        OperandType::BYTE => {
            let byte = get_byte_from_register(return_register);

            Ok(Some(Value::Byte(byte)))
        }
        OperandType::CHARACTER => {
            let character = get_character_from_register(return_register);

            Ok(Some(Value::Character(character)))
        }
        OperandType::FLOAT => {
            let float = get_float_from_register(return_register);

            Ok(Some(Value::Float(float)))
        }
        OperandType::INTEGER => {
            let integer = get_integer_from_register(return_register);

            Ok(Some(Value::Integer(integer)))
        }
        OperandType::STRING => {
            let string = get_string_from_register(return_register)?;

            Ok(Some(Value::String(string)))
        }
        OperandType::LIST_BOOLEAN
        | OperandType::LIST_BYTE
        | OperandType::LIST_CHARACTER
        | OperandType::LIST_FLOAT
        | OperandType::LIST_INTEGER
        | OperandType::LIST_STRING
        | OperandType::LIST_LIST
        | OperandType::LIST_FUNCTION => {
            let Some(TypeNode::Function(FunctionTypeNode { return_type, .. })) = program
                .resolver
                .get_type_node(program.main_chunk().type_id)
                .copied()
            else {
                unreachable!("Main chunk type must be a function");
            };
            let Some(list_type) = program.resolver.resolve_type(return_type) else {
                unreachable!("Main chunk return type must be resolvable");
            };
            let list = get_list_from_register(return_register, &list_type)?;

            Ok(Some(Value::List(list)))
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
    pub const CRANELIFT_TYPE: CraneliftType = match size_of::<ThreadResult>() {
        4 => I32,
        8 => I64,
        _ => panic!("Unsupported ThreadStatus size"),
    };
}

#[repr(C)]
pub struct ThreadContext<'a> {
    pub call_stack_vec_pointer: *mut Vec<u8>,
    pub call_stack_buffer_pointer: *mut u8,
    pub call_stack_allocated_length_pointer: *mut usize,
    pub call_stack_used_length_pointer: *mut usize,

    pub register_stack_vec_pointer: *mut Vec<Register>,
    pub register_stack_buffer_pointer: *mut Register,
    pub register_stack_allocated_length_pointer: *mut usize,
    pub register_stack_used_length_pointer: *mut usize,

    pub register_tags_vec_pointer: *mut Vec<RegisterTag>,
    pub register_tags_buffer_pointer: *mut RegisterTag,

    pub object_pool_pointer: *mut ObjectPool<'a>,

    pub return_register_pointer: *mut Register,
    pub return_type_pointer: *mut OperandType,
}

fn get_boolean_from_register(register: Register) -> bool {
    unsafe { register.boolean }
}

fn get_byte_from_register(register: Register) -> u8 {
    unsafe { register.byte }
}

fn get_character_from_register(register: Register) -> char {
    unsafe { register.character }
}

fn get_float_from_register(register: Register) -> f64 {
    unsafe { register.float }
}

fn get_integer_from_register(register: Register) -> i64 {
    unsafe { register.integer }
}

fn get_string_from_register(register: Register) -> Result<String, JitError> {
    let object_pointer = unsafe { register.object_pointer };
    let object = unsafe { &*object_pointer };

    object
        .as_string()
        .cloned()
        .ok_or(JitError::InvalidConstantType {
            expected_type: OperandType::STRING,
        })
}

fn get_list_from_register(register: Register, full_type: &Type) -> Result<List, JitError> {
    let object_pointer = unsafe { register.object_pointer };
    let object = unsafe { &*object_pointer };

    match &object.value {
        ObjectValue::BooleanList(booleans) => Ok(List::Boolean(booleans.clone())),
        ObjectValue::ByteList(bytes) => Ok(List::Byte(bytes.clone())),
        ObjectValue::CharacterList(characters) => Ok(List::Character(characters.clone())),
        ObjectValue::FloatList(floats) => Ok(List::Float(floats.clone())),
        ObjectValue::IntegerList(integers) => Ok(List::Integer(integers.clone())),
        ObjectValue::ObjectList(objects) => {
            let item_type = if let Type::List(item_type) = full_type {
                item_type.as_ref()
            } else {
                return Err(JitError::InvalidConstantType {
                    expected_type: full_type.as_operand_type(),
                });
            };

            if item_type == &Type::String {
                let mut strings = Vec::with_capacity(objects.len());

                for object_pointer in objects {
                    let object = unsafe { &**object_pointer };
                    let string = match &object.value {
                        ObjectValue::String(string) => string.clone(),
                        _ => {
                            return Err(JitError::InvalidConstantType {
                                expected_type: OperandType::LIST_STRING,
                            });
                        }
                    };

                    strings.push(string);
                }

                return Ok(List::String(strings));
            }

            let mut items = Vec::with_capacity(objects.len());

            for object_pointer in objects {
                let object = unsafe { &**object_pointer };
                let value = match &object.value {
                    ObjectValue::BooleanList(boolean_list) => List::Boolean(boolean_list.clone()),
                    ObjectValue::ByteList(byte_list) => List::Byte(byte_list.clone()),
                    ObjectValue::CharacterList(character_list) => {
                        List::Character(character_list.clone())
                    }
                    ObjectValue::FloatList(float_list) => List::Float(float_list.clone()),
                    ObjectValue::IntegerList(integer_list) => List::Integer(integer_list.clone()),
                    ObjectValue::ObjectList(object_list) => {
                        let mut inner_lists = Vec::with_capacity(object_list.len());

                        for inner_object_pointer in object_list {
                            let inner_list_type = if let Type::List(inner_item_type) = item_type {
                                inner_item_type.as_ref()
                            } else {
                                return Err(JitError::InvalidConstantType {
                                    expected_type: item_type.as_operand_type(),
                                });
                            };

                            let inner_list = get_list_from_register(
                                Register {
                                    object_pointer: *inner_object_pointer as *mut _,
                                },
                                inner_list_type,
                            )?;

                            inner_lists.push(inner_list);
                        }

                        List::List(inner_lists)
                    }
                    _ => {
                        return Err(JitError::InvalidConstantType {
                            expected_type: item_type.as_operand_type(),
                        });
                    }
                };

                items.push(value);
            }

            Ok(List::List(items))
        }
        _ => Err(JitError::InvalidConstantType {
            expected_type: OperandType::LIST_BOOLEAN,
        }),
    }
}
