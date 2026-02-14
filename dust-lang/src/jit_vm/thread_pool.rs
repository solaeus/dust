use std::{
    collections::HashMap,
    mem::offset_of,
    sync::{Arc, Mutex, MutexGuard},
    thread::{self, Builder as ThreadBuilder, JoinHandle, ThreadId},
};

use bumpalo::Bump;
use cranelift::prelude::{
    FunctionBuilder, InstBuilder, MemFlags, Type as CraneliftType, Value as CraneliftValue,
    types::{I8, I64},
};
use crossbeam_channel::{Receiver, Sender};
use rustc_hash::FxBuildHasher;
use tracing::{Level, debug, info, span};

use crate::{
    dust_crate::Program,
    dust_type::{DustStructType, DustType},
    instruction::ByteType,
    jit_vm::{
        JitCompiler, JitError, JitFunction, Object, ObjectPool, Register, RegisterTag,
        object::ObjectValue,
    },
    value::{List, Value},
};

pub struct ThreadPool {
    spawner: Arc<Mutex<ThreadSpawner>>,
}

impl ThreadPool {
    pub fn new(
        program: Arc<Program>,
        minimum_object_heap: usize,
        minimum_object_sweep: usize,
    ) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();

        ThreadPool {
            spawner: Arc::new(Mutex::new(ThreadSpawner {
                program,
                threads: HashMap::default(),
                message_sender: Arc::new(sender),
                message_receiver: receiver,
                minimum_object_heap,
                minimum_object_sweep,
            })),
        }
    }

    pub fn lock_spawner(&self) -> MutexGuard<'_, ThreadSpawner> {
        self.spawner
            .lock()
            .expect("Failed to lock ThreadSpawner mutex")
    }

    pub fn clone_spawner(&self) -> Arc<Mutex<ThreadSpawner>> {
        Arc::clone(&self.spawner)
    }
}

pub struct ThreadSpawner {
    program: Arc<Program>,

    threads: HashMap<ThreadId, JoinHandle<()>, FxBuildHasher>,

    message_sender: Arc<Sender<ThreadMessage>>,
    message_receiver: Receiver<ThreadMessage>,

    minimum_object_heap: usize,
    minimum_object_sweep: usize,
}

impl ThreadSpawner {
    pub fn is_empty(&self) -> bool {
        self.threads.is_empty()
    }

    pub fn spawn_thread(
        &mut self,
        prototype_id: u16,
        spawner: Arc<Mutex<ThreadSpawner>>,
    ) -> Result<(), JitError> {
        let message_sender = Arc::clone(&self.message_sender);
        let program = Arc::clone(&self.program);
        let minimum_object_heap = self.minimum_object_heap;
        let minimum_object_sweep = self.minimum_object_sweep;
        let join_handle = ThreadBuilder::new()
            .spawn(move || {
                let result = run_thread(
                    program,
                    prototype_id,
                    minimum_object_heap,
                    minimum_object_sweep,
                    spawner,
                );
                let thread_message = ThreadMessage::RemoveThread {
                    thread_id: thread::current().id(),
                    result,
                    prototype_id,
                };

                message_sender
                    .send(thread_message)
                    .expect("Failed to send thread message");
            })
            .expect("Failed to spawn thread");

        self.threads.insert(join_handle.thread().id(), join_handle);

        Ok(())
    }

    pub fn spawn_named_thread(
        &mut self,
        thread_name: String,
        prototype_id: u16,
        spawner: Arc<Mutex<ThreadSpawner>>,
    ) -> Result<(), JitError> {
        let message_sender = Arc::clone(&self.message_sender);
        let program = Arc::clone(&self.program);
        let minimum_object_heap = self.minimum_object_heap;
        let minimum_object_sweep = self.minimum_object_sweep;
        let join_handle = ThreadBuilder::new()
            .name(thread_name)
            .spawn(move || {
                let result = run_thread(
                    program,
                    prototype_id,
                    minimum_object_heap,
                    minimum_object_sweep,
                    spawner,
                );
                let thread_message = ThreadMessage::RemoveThread {
                    thread_id: thread::current().id(),
                    result,
                    prototype_id,
                };

                message_sender
                    .send(thread_message)
                    .expect("Failed to send thread message");
            })
            .expect("Failed to spawn thread");

        self.threads.insert(join_handle.thread().id(), join_handle);

        Ok(())
    }

    pub fn clone_message_receiver(&self) -> Receiver<ThreadMessage> {
        self.message_receiver.clone()
    }

    pub fn threads_mut(&mut self) -> &mut HashMap<ThreadId, JoinHandle<()>, FxBuildHasher> {
        &mut self.threads
    }
}

pub enum ThreadMessage {
    SpawnThread {
        thread_name: String,
        prototype_id: u16,
    },
    RemoveThread {
        thread_id: ThreadId,
        result: Result<Option<Value>, JitError>,
        prototype_id: u16,
    },
}

#[repr(C)]
pub struct ThreadStatus(u8);

impl ThreadStatus {
    pub const OK: Self = Self(0);
    pub const ERROR_LIST_INDEX_OUT_OF_BOUNDS: Self = Self(1);
    pub const ERROR_DIVISION_BY_ZERO: Self = Self(2);

    pub const CRANELIFT_TYPE: CraneliftType = I8;
}

#[repr(C)]
#[derive(Clone, Default)]
pub struct JitPrototype {
    pub function_pointer: *mut u8,
    pub return_value_tags_vec: Vec<RegisterTag>,
    pub return_value_tags_buffer: *const RegisterTag,
    pub return_value_count: usize,
    pub return_kind: u8, // 0 = none, 1 = scalar/object (i64), 2 = struct (sret)
    pub is_recursive: bool,
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
    pub thread_spawner_pointer: *const Arc<Mutex<ThreadSpawner>>,

    pub jit_prototype_buffer_pointer: *mut JitPrototype,

    pub function_arguments: [i64; 10],

    pub recursive_return_register: i64,
}

impl<'a> ThreadContext<'a> {
    pub fn get_fields(
        thread_context: CraneliftValue,
        pointer_type: CraneliftType,
        builder: &mut FunctionBuilder,
    ) -> ThreadContextFields {
        fn get_field(
            field_type: CraneliftType,
            offset: usize,
            thread_context: &CraneliftValue,
            builder: &mut FunctionBuilder<'_>,
        ) -> CraneliftValue {
            builder
                .ins()
                .load(field_type, MemFlags::new(), *thread_context, offset as i32)
        }

        ThreadContextFields {
            status: get_field(
                ThreadStatus::CRANELIFT_TYPE,
                offset_of!(ThreadContext, status),
                &thread_context,
                builder,
            ),
            register_vec_pointer: get_field(
                pointer_type,
                offset_of!(ThreadContext, register_vec_pointer),
                &thread_context,
                builder,
            ),
            register_buffer_pointer: get_field(
                pointer_type,
                offset_of!(ThreadContext, register_buffer_pointer),
                &thread_context,
                builder,
            ),
            register_tag_vec_pointer: get_field(
                pointer_type,
                offset_of!(ThreadContext, register_tag_vec_pointer),
                &thread_context,
                builder,
            ),
            register_tag_buffer_pointer: get_field(
                pointer_type,
                offset_of!(ThreadContext, register_tag_buffer_pointer),
                &thread_context,
                builder,
            ),
            registers_allocated: get_field(
                I64,
                offset_of!(ThreadContext, registers_allocated),
                &thread_context,
                builder,
            ),
            registers_used: get_field(
                I64,
                offset_of!(ThreadContext, registers_used),
                &thread_context,
                builder,
            ),
            object_pool_pointer: get_field(
                pointer_type,
                offset_of!(ThreadContext, object_pool_pointer),
                &thread_context,
                builder,
            ),
            thread_pool_pointer: get_field(
                pointer_type,
                offset_of!(ThreadContext, thread_spawner_pointer),
                &thread_context,
                builder,
            ),
            jit_prototype_buffer_pointer: get_field(
                pointer_type,
                offset_of!(ThreadContext, jit_prototype_buffer_pointer),
                &thread_context,
                builder,
            ),
            function_arguments: builder.ins().iadd_imm(
                thread_context,
                offset_of!(ThreadContext, function_arguments) as i64,
            ),
            recursive_return_register: get_field(
                I64,
                offset_of!(ThreadContext, recursive_return_register),
                &thread_context,
                builder,
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
    pub thread_pool_pointer: CraneliftValue,
    pub jit_prototype_buffer_pointer: CraneliftValue,
    pub function_arguments: CraneliftValue,
    pub recursive_return_register: CraneliftValue,
}

fn run_thread(
    program: Arc<Program>,
    prototype_id: u16,
    minimum_object_heap: usize,
    minimum_object_sweep: usize,
    thread_spawner: Arc<Mutex<ThreadSpawner>>,
) -> Result<Option<Value>, JitError> {
    let span = span!(Level::INFO, "run_thread");
    let _enter = span.enter();

    info!("Starting JIT compilation for proto_{prototype_id}");

    let mut jit = JitCompiler::new(&program)?;
    let (jit_function, mut jit_prototypes) = jit.compile()?;

    info!("JIT compilation complete");

    let registers_allocated = 1024;
    let mut registers = vec![Register { integer: 0 }; registers_allocated];
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
        thread_spawner_pointer: &thread_spawner,
        jit_prototype_buffer_pointer: jit_prototypes.as_mut_ptr(),
        function_arguments: [0; 10],
        status: ThreadStatus::OK,
        recursive_return_register: 0,
    };

    let return_register_count = jit_prototypes
        .get(prototype_id as usize)
        .map(|prototype| prototype.return_value_count)
        .unwrap_or(1)
        .max(1);
    let mut return_registers = vec![0_i64; return_register_count];

    match jit_function {
        JitFunction::None(logic) => {
            logic(&mut thread_context, 0);
        }
        JitFunction::Scalar(logic) => {
            return_registers[0] = logic(&mut thread_context, 0);
        }
        JitFunction::Struct(logic) => {
            logic(return_registers.as_mut_ptr(), &mut thread_context, 0);
        }
    }

    let return_type = &program.prototypes[prototype_id as usize].return_type;

    if return_type == &DustType::None {
        return Ok(None);
    }

    let (return_value, _) = decode_value_from_return_words(return_type, &return_registers, 0)?;

    info!("JIT execution completed, returning {return_value:?} with type {return_type}");
    debug!("{}", object_pool.report());

    Ok(Some(return_value))
}

fn get_list_from_object_index(
    object_pointer: *mut Object,
    full_type: &DustType,
) -> Result<List, JitError> {
    let object = unsafe { object_pointer.as_ref().ok_or(JitError::MissingReturnValue) }?;

    match &object.value {
        ObjectValue::BooleanList(booleans) => Ok(List::Boolean(booleans.clone())),
        ObjectValue::ByteList(bytes) => Ok(List::Byte(bytes.clone())),
        ObjectValue::CharacterList(characters) => Ok(List::Character(characters.clone())),
        ObjectValue::FloatList(floats) => Ok(List::Float(floats.clone())),
        ObjectValue::IntegerList(integers) => Ok(List::Integer(integers.clone())),
        ObjectValue::ObjectList(objects) => {
            let item_type = if let DustType::List(item_type) = full_type {
                item_type.as_ref()
            } else {
                return Err(JitError::InvalidConstantType {
                    expected_type: full_type.as_operand_type(),
                });
            };

            if item_type == &DustType::String {
                let mut strings = Vec::with_capacity(objects.len());

                for object_pointer in objects {
                    let object = unsafe {
                        object_pointer
                            .as_ref()
                            .ok_or(JitError::MissingReturnValue)?
                    };
                    let string = match &object.value {
                        ObjectValue::String(string) => string.clone(),
                        _ => {
                            return Err(JitError::InvalidObjectValue {
                                expected: ByteType::STRING,
                            });
                        }
                    };

                    strings.push(string);
                }

                return Ok(List::String(strings));
            }

            let mut items = Vec::with_capacity(objects.len());

            for object_pointer in objects {
                let object = unsafe {
                    object_pointer
                        .as_ref()
                        .ok_or(JitError::MissingReturnValue)?
                };
                let list = match &object.value {
                    ObjectValue::BooleanList(boolean_list) => List::Boolean(boolean_list.clone()),
                    ObjectValue::ByteList(byte_list) => List::Byte(byte_list.clone()),
                    ObjectValue::CharacterList(character_list) => {
                        List::Character(character_list.clone())
                    }
                    ObjectValue::FloatList(float_list) => List::Float(float_list.clone()),
                    ObjectValue::IntegerList(integer_list) => List::Integer(integer_list.clone()),
                    ObjectValue::ObjectList(object_list) => {
                        let mut inner_lists = Vec::with_capacity(object_list.len());

                        for object_pointer in object_list {
                            let inner_list_type = if let DustType::List(inner_item_type) = item_type
                            {
                                inner_item_type.as_ref()
                            } else {
                                return Err(JitError::InvalidObjectType {
                                    expected: item_type.clone(),
                                });
                            };
                            let inner_list =
                                get_list_from_object_index(*object_pointer, inner_list_type)?;

                            inner_lists.push(inner_list);
                        }

                        List::Nested(inner_lists)
                    }
                    _ => {
                        return Err(JitError::InvalidConstantType {
                            expected_type: item_type.as_operand_type(),
                        });
                    }
                };

                items.push(list);
            }

            Ok(List::Nested(items))
        }
        _ => Err(JitError::InvalidConstantType {
            expected_type: ByteType::LIST_BOOLEAN,
        }),
    }
}

fn decode_value_from_return_words(
    r#type: &DustType,
    return_words: &[i64],
    start: usize,
) -> Result<(Value, usize), JitError> {
    match r#type {
        DustType::None => Err(JitError::MissingReturnValue),
        DustType::Boolean => Ok((Value::Boolean(return_words[start] != 0), 1)),
        DustType::Byte => Ok((Value::Byte(return_words[start] as u8), 1)),
        DustType::Character => Ok((
            Value::Character(char::from_u32(return_words[start] as u32).unwrap_or_default()),
            1,
        )),
        DustType::Float => Ok((Value::Float(f64::from_bits(return_words[start] as u64)), 1)),
        DustType::Integer => Ok((Value::Integer(return_words[start]), 1)),
        DustType::String => {
            let string = unsafe { (return_words[start] as *const Object).as_ref() }
                .ok_or(JitError::MissingReturnValue)?
                .as_string()
                .unwrap()
                .clone();

            Ok((Value::String(string), 1))
        }
        DustType::List(_) => {
            let object_pointer = return_words[start] as *mut Object;
            let list = get_list_from_object_index(object_pointer, r#type)?;

            Ok((Value::List(list), 1))
        }
        DustType::Struct(struct_type) => {
            let DustStructType { name, fields } = struct_type.as_ref();

            let mut offset = start;
            let mut field_values = Vec::with_capacity(fields.len());

            for (field_name, field_type) in fields {
                let (value, consumed) =
                    decode_value_from_return_words(field_type, return_words, offset)?;
                offset += consumed;

                field_values.push((field_name.clone(), value));
            }

            Ok((
                Value::Struct {
                    name: name.clone(),
                    fields: field_values,
                },
                offset - start,
            ))
        }
        DustType::Function(_) => todo!("Error"),
    }
}
