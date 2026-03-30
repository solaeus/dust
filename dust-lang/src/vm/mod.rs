mod call_frame;
pub mod error;
mod object;
mod object_pool;
mod register;
mod thread;
mod thread_pool;

use std::sync::Arc;

use tracing::{Level, error, info, span};

use crate::{
    compiler::Compiler,
    dust_type::DustType,
    dust_value::DustValue,
    error::{Error, ErrorKind},
    program::Program,
    source::{Source, SourceFile},
    vm::{
        error::VmError,
        thread_pool::{ThreadMessage, ThreadPool},
    },
};

pub const MINIMUM_OBJECT_HEAP_DEFAULT: usize = if cfg!(debug_assertions) {
    1024
} else {
    1024 * 1024 * 4
};
pub const MINIMUM_OBJECT_SWEEP_DEFAULT: usize = if cfg!(debug_assertions) {
    256
} else {
    1024 * 1024
};

pub fn run<'src>(source_code: &'src str) -> Result<Option<DustValue>, Error<'src>> {
    let mut source = Source::new();

    source.add_file(SourceFile::validated_borrowed("eval", source_code));

    let compiler = Compiler::new(source);
    let program = compiler.compile(None)?;
    let vm = Vm::new(
        program,
        MINIMUM_OBJECT_HEAP_DEFAULT,
        MINIMUM_OBJECT_SWEEP_DEFAULT,
    );

    vm.run()
}

pub struct Vm {
    program: Arc<Program>,
    thread_pool: ThreadPool,
}

impl Vm {
    pub fn new(program: Program, minimum_object_heap: usize, minimum_object_sweep: usize) -> Self {
        let program = Arc::new(program);

        Self {
            program: Arc::clone(&program),
            thread_pool: ThreadPool::new(program, minimum_object_heap, minimum_object_sweep),
        }
    }

    pub fn run<'a>(self) -> Result<Option<DustValue>, Error<'a>> {
        let span = span!(Level::INFO, "run");
        let _enter = span.enter();

        let message_receiver = {
            info!("Spawning main VM thread");

            let mut local_spawner = self.thread_pool.lock_spawner();

            local_spawner
                .spawn_thread(0)
                .map_err(|error| Error::without_context(vec![ErrorKind::Vm(error)]))?;
            local_spawner.clone_message_receiver()
        };

        let mut return_value: Option<DustValue> = None;

        loop {
            match message_receiver.recv() {
                Ok(ThreadMessage::SpawnThread {
                    thread_name,
                    prototype_id: prototype_index,
                }) => {
                    info!("Spawning VM thread: {thread_name} with proto_{prototype_index}");

                    self.thread_pool
                        .lock_spawner()
                        .spawn_named_thread(thread_name, prototype_index)
                        .map_err(|error| Error::without_context(vec![ErrorKind::Vm(error)]))?;
                }
                Ok(ThreadMessage::RemoveThread { thread_id, result }) => {
                    info!("VM thread completed: Thread ID: {}", thread_id.as_u64());

                    let mut spawner = self.thread_pool.lock_spawner();
                    let removed_handle = spawner.threads_mut().remove(&thread_id);

                    if let Some(handle) = removed_handle {
                        if handle.is_finished() {
                            info!(
                                "Thread {} was finished and removed from the thread pool.",
                                thread_id.as_u64()
                            );
                        } else {
                            error!(
                                "Thread {} was not finished when removed from the thread pool.",
                                thread_id.as_u64()
                            );
                        }
                    } else {
                        error!("Failed to remove VM thread.");
                    }

                    if spawner.is_empty() {
                        info!("All VM threads have completed.");

                        let return_registers = result
                            .map_err(|error| Error::without_context(vec![ErrorKind::Vm(error)]))?;

                        return_value = self
                            .create_return_value(return_registers)
                            .map_err(|error| Error::without_context(vec![ErrorKind::Vm(error)]))?;

                        break;
                    }
                }
                Err(error) => {
                    error!("VM Error: {}", error);

                    break;
                }
            }
        }

        Ok(return_value)
    }

    fn create_return_value(
        &self,
        return_registers: Vec<register::Register>,
    ) -> Result<Option<DustValue>, VmError> {
        match self.program.return_type() {
            DustType::Unit => {
                if return_registers.is_empty() {
                    return Ok(None);
                }
            }
            DustType::Boolean => {
                if return_registers.len() == 1 {
                    return Ok(Some(DustValue::Boolean(return_registers[0].0 != 0)));
                }
            }
            DustType::Character => {
                if return_registers.len() == 1 {
                    let value = return_registers[0].0;
                    let character = char::from_u32(value).unwrap_or_default();

                    return Ok(Some(DustValue::Character(character)));
                }
            }
            DustType::U8 => {
                if return_registers.len() == 1 {
                    let value = return_registers[0].0;

                    return Ok(Some(DustValue::U8(value as u8)));
                }
            }
            DustType::U16 => {
                if return_registers.len() == 1 {
                    let value = return_registers[0].0;

                    return Ok(Some(DustValue::U16(value as u16)));
                }
            }
            DustType::U32 => {
                if return_registers.len() == 1 {
                    let value = return_registers[0].0;

                    return Ok(Some(DustValue::U32(value)));
                }
            }
            DustType::U64 => {
                if return_registers.len() == 2 {
                    let low = return_registers[0].0 as u64;
                    let high = return_registers[1].0 as u64;
                    let value = (high << 32) | low;

                    return Ok(Some(DustValue::U64(value)));
                }
            }
            DustType::U128 => {
                if return_registers.len() == 4 {
                    let value_0 = return_registers[0].0 as u128;
                    let value_1 = return_registers[1].0 as u128;
                    let value_2 = return_registers[2].0 as u128;
                    let value_3 = return_registers[3].0 as u128;
                    let value = (value_3 << 96) | (value_2 << 64) | (value_1 << 32) | value_0;

                    return Ok(Some(DustValue::U128(value)));
                }
            }
            DustType::I8 => {
                if return_registers.len() == 1 {
                    let value = return_registers[0].0;

                    return Ok(Some(DustValue::I8(value as i8)));
                }
            }
            DustType::I16 => {
                if return_registers.len() == 1 {
                    let value = return_registers[0].0;

                    return Ok(Some(DustValue::I16(value as i16)));
                }
            }
            DustType::I32 => {
                if return_registers.len() == 1 {
                    let value = return_registers[0].0;

                    return Ok(Some(DustValue::I32(value as i32)));
                }
            }
            DustType::I64 => {
                if return_registers.len() == 2 {
                    let low = return_registers[0].0 as u64;
                    let high = return_registers[1].0 as u64;
                    let value = (high << 32) | low;

                    return Ok(Some(DustValue::I64(value as i64)));
                }
            }
            DustType::I128 => {
                if return_registers.len() == 4 {
                    let value_0 = return_registers[0].0 as u128;
                    let value_1 = return_registers[1].0 as u128;
                    let value_2 = return_registers[2].0 as u128;
                    let value_3 = return_registers[3].0 as u128;
                    let value = (value_3 << 96) | (value_2 << 64) | (value_1 << 32) | value_0;

                    return Ok(Some(DustValue::I128(value as i128)));
                }
            }
            DustType::F32 => {
                if return_registers.len() == 1 {
                    let value = return_registers[0].0;
                    let float_value = f32::from_bits(value);

                    return Ok(Some(DustValue::F32(float_value)));
                }
            }
            DustType::F64 => {
                if return_registers.len() == 2 {
                    let low = return_registers[0].0 as u64;
                    let high = return_registers[1].0 as u64;
                    let value = (high << 32) | low;
                    let float_value = f64::from_bits(value);

                    return Ok(Some(DustValue::F64(float_value)));
                }
            }
            DustType::Tuple(dust_type) => todo!(),
            DustType::Array(dust_type, _) => todo!(),
            DustType::Slice(dust_type) => todo!(),
            DustType::Function(dust_function_type) => todo!(),
            DustType::Struct(dust_struct_type) => todo!(),
            DustType::Enum(enum_name, variant_types) => {
                if return_registers.is_empty() {
                    return Err(VmError::InvalidReturnValue {
                        register_count: 0,
                        expected_type: self.program.return_type().clone(),
                    });
                }

                let discriminant = return_registers[0].0 as usize;

                if discriminant >= variant_types.len() {
                    return Err(VmError::InvalidReturnValue {
                        register_count: return_registers.len(),
                        expected_type: self.program.return_type().clone(),
                    });
                }

                let variant = &variant_types[discriminant];
                let variant_name = variant.name.clone();
                let mut fields = Vec::new();
                let mut register_index = 1;

                for (_, field_type) in &variant.fields {
                    let field_value = match field_type {
                        DustType::Boolean => {
                            let value = return_registers[register_index].0;

                            register_index += 1;

                            DustValue::Boolean(value != 0)
                        }
                        DustType::U8 => {
                            let value = return_registers[register_index].0;

                            register_index += 1;

                            DustValue::U8(value as u8)
                        }
                        DustType::U16 => {
                            let value = return_registers[register_index].0;

                            register_index += 1;

                            DustValue::U16(value as u16)
                        }
                        DustType::U32 => {
                            let value = return_registers[register_index].0;

                            register_index += 1;

                            DustValue::U32(value)
                        }
                        DustType::U64 => {
                            let low = return_registers[register_index].0 as u64;
                            let high = return_registers[register_index + 1].0 as u64;
                            let value = (high << 32) | low;

                            register_index += 2;

                            DustValue::U64(value)
                        }
                        DustType::I8 => {
                            let value = return_registers[register_index].0;

                            register_index += 1;

                            DustValue::I8(value as i8)
                        }
                        DustType::I16 => {
                            let value = return_registers[register_index].0;

                            register_index += 1;

                            DustValue::I16(value as i16)
                        }
                        DustType::I32 => {
                            let value = return_registers[register_index].0;

                            register_index += 1;

                            DustValue::I32(value as i32)
                        }
                        DustType::I64 => {
                            let low = return_registers[register_index].0 as u64;
                            let high = return_registers[register_index + 1].0 as u64;
                            let value = (high << 32) | low;

                            register_index += 2;

                            DustValue::I64(value as i64)
                        }
                        DustType::F32 => {
                            let value = return_registers[register_index].0;

                            register_index += 1;

                            DustValue::F32(f32::from_bits(value))
                        }
                        DustType::F64 => {
                            let low = return_registers[register_index].0 as u64;
                            let high = return_registers[register_index + 1].0 as u64;
                            let value = (high << 32) | low;

                            register_index += 2;

                            DustValue::F64(f64::from_bits(value))
                        }
                        _ => {
                            return Err(VmError::InvalidReturnValue {
                                register_count: return_registers.len(),
                                expected_type: self.program.return_type().clone(),
                            });
                        }
                    };

                    fields.push(field_value);
                }

                return Ok(Some(DustValue::Enum {
                    enum_name: enum_name.clone(),
                    variant_name,
                    fields,
                }));
            }
        }

        Err(VmError::InvalidReturnValue {
            register_count: return_registers.len(),
            expected_type: self.program.return_type().clone(),
        })
    }
}
