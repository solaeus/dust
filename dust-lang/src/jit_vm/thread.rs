use std::{
    mem::offset_of,
    sync::Arc,
    thread::{Builder as ThreadBuilder, JoinHandle},
};

use bumpalo::Bump;
use cranelift::prelude::{
    FunctionBuilder, InstBuilder, MemFlags, Type as CraneliftType, Value as CraneliftValue,
    types::{I32, I64},
};
use tracing::{Level, debug, info, span};

use crate::{
    dust_crate::Program,
    instruction::OperandType,
    jit_vm::{
        JitError, Object, ObjectPool, Register, RegisterTag, jit_compiler::JitCompiler,
        object::ObjectValue,
    },
    r#type::Type,
    value::{List, Value},
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
            .spawn(move || run(program, minimum_object_heap, minimum_object_sweep))
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

    let mut jit = JitCompiler::new(&program)?;
    let jit_logic = jit.compile()?;

    info!("JIT compilation complete");

    let registers_allocated = if program.prototypes.len() == 1 {
        program.prototypes[0].register_count as usize
    } else {
        1024
    };
    let mut registers = vec![Register { empty: () }; registers_allocated];
    let mut register_tags = vec![RegisterTag::EMPTY; registers_allocated];
    let bump_arena = Bump::with_capacity(minimum_object_heap);
    let mut object_pool = ObjectPool::new(&bump_arena, minimum_object_sweep, minimum_object_heap);

    let mut thread_context = ThreadContext {
        register_vec_pointer: &mut registers,
        register_buffer_pointer: registers.as_mut_ptr(),
        register_tag_vec_pointer: &mut register_tags,
        register_tag_buffer_pointer: register_tags.as_mut_ptr(),
        registers_allocated,
        registers_used: 0,
        object_pool_pointer: &mut object_pool,
        status: ThreadStatus::Ok,
        recursive_return_register: 0,
    };

    let encoded_return_value = (jit_logic)(&mut thread_context, 0);
    let return_type = &program.prototypes[0].function_type.return_type;
    let return_value = match *return_type {
        Type::None => None,
        Type::Boolean => {
            let boolean = encoded_return_value != 0;

            Some(Value::Boolean(boolean))
        }
        Type::Byte => {
            let byte = encoded_return_value as u8;

            Some(Value::Byte(byte))
        }
        Type::Character => {
            let character = char::from_u32(encoded_return_value as u32).unwrap_or_default();

            Some(Value::Character(character))
        }
        Type::Float => {
            let float = f64::from_bits(encoded_return_value as u64);

            Some(Value::Float(float))
        }
        Type::Integer => Some(Value::Integer(encoded_return_value)),
        Type::String => {
            let string = {
                let object_pointer = encoded_return_value as *const Object;
                let object = unsafe { &*object_pointer };

                object
                    .as_string()
                    .cloned()
                    .ok_or(JitError::InvalidConstantType {
                        expected_type: OperandType::STRING,
                    })
            }?;

            Some(Value::String(string))
        }
        Type::List(_) => {
            let object_pointer = encoded_return_value as *const Object;
            let list = get_list_from_object_pointer(object_pointer, return_type)?;

            Some(Value::List(list))
        }
        Type::Function(_) => todo!("Error"),
    };

    info!("JIT execution completed, returning {return_value:?} with type {return_type}");
    debug!("{}", object_pool.report());

    Ok(return_value)
}

#[repr(C)]
pub enum ThreadStatus {
    Ok = 0,
    ErrorDivisionByZero = 1,
    ErrorListIndexOutOfBounds = 2,
}

impl ThreadStatus {
    pub const CRANELIFT_TYPE: CraneliftType = match size_of::<ThreadStatus>() {
        4 => I32,
        8 => I64,
        _ => panic!("Unsupported ThreadStatus size"),
    };
}

#[repr(C)]
pub struct ThreadContext<'a> {
    pub status: ThreadStatus,

    pub register_vec_pointer: *mut Vec<Register>,
    pub register_buffer_pointer: *mut Register,

    pub register_tag_vec_pointer: *mut Vec<RegisterTag>,
    pub register_tag_buffer_pointer: *mut RegisterTag,

    pub registers_allocated: usize,
    pub registers_used: usize,

    pub object_pool_pointer: *mut ObjectPool<'a>,

    pub recursive_return_register: i64,
}

impl<'a> ThreadContext<'a> {
    pub fn get_fields(
        thread_context: CraneliftValue,
        pointer_type: CraneliftType,
        builder: &mut FunctionBuilder,
    ) -> ThreadContextFields {
        let mut get_field = |field_type: CraneliftType, offset: usize| {
            builder
                .ins()
                .load(field_type, MemFlags::new(), thread_context, offset as i32)
        };

        ThreadContextFields {
            status: get_field(
                ThreadStatus::CRANELIFT_TYPE,
                offset_of!(ThreadContext, status),
            ),
            register_vec_pointer: get_field(
                pointer_type,
                offset_of!(ThreadContext, register_vec_pointer),
            ),
            register_buffer_pointer: get_field(
                pointer_type,
                offset_of!(ThreadContext, register_buffer_pointer),
            ),
            register_tag_vec_pointer: get_field(
                pointer_type,
                offset_of!(ThreadContext, register_tag_vec_pointer),
            ),
            register_tag_buffer_pointer: get_field(
                pointer_type,
                offset_of!(ThreadContext, register_tag_buffer_pointer),
            ),
            registers_allocated: get_field(I64, offset_of!(ThreadContext, registers_allocated)),
            registers_used: get_field(I64, offset_of!(ThreadContext, registers_used)),
            object_pool_pointer: get_field(
                pointer_type,
                offset_of!(ThreadContext, object_pool_pointer),
            ),
            recursive_return_register: get_field(
                I64,
                offset_of!(ThreadContext, recursive_return_register),
            ),
        }
    }
}

pub struct ThreadContextFields {
    pub status: CraneliftValue,

    pub register_vec_pointer: CraneliftValue,
    pub register_buffer_pointer: CraneliftValue,

    pub register_tag_vec_pointer: CraneliftValue,
    pub register_tag_buffer_pointer: CraneliftValue,

    pub registers_allocated: CraneliftValue,
    pub registers_used: CraneliftValue,

    pub object_pool_pointer: CraneliftValue,

    pub recursive_return_register: CraneliftValue,
}

fn get_list_from_object_pointer(
    object_pointer: *const Object,
    full_type: &Type,
) -> Result<List, JitError> {
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
                            return Err(JitError::InvalidObjectValue {
                                expected: OperandType::STRING,
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
                                return Err(JitError::InvalidObjectType {
                                    expected: item_type.clone(),
                                });
                            };

                            let inner_list = get_list_from_object_pointer(
                                *inner_object_pointer,
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
