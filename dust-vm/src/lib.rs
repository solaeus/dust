mod call;
pub mod error;
mod object;
mod object_pool;
mod register;
mod thread;
mod thread_pool;

use std::sync::Arc;

use tracing::{Level, error, info, span};

use dust_compiler::{
    compiler::Compiler,
    dust_type::{DustEnumType, DustStructType, DustStructTypeFields, DustType},
    dust_value::{DustEnumVariant, DustStruct, DustStructValue, DustValue},
    error::{Error, ErrorContext, ErrorKind},
    program::Program,
    source::{Source, SourceCode},
    vm::{
        error::VmError,
        register::Register,
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

    source.add_code(SourceCode::validated("eval", source_code));

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

            let mut spawner = self.thread_pool.lock_spawner();

            spawner
                .spawn_thread(0)
                .map_err(|error| Error::new(vec![ErrorKind::Vm(error)], ErrorContext::None))?;
            spawner.clone_message_receiver()
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
                        .map_err(|error| {
                            Error::new(vec![ErrorKind::Vm(error)], ErrorContext::None)
                        })?;
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

                        let return_registers = result.map_err(|error| {
                            Error::new(vec![ErrorKind::Vm(error)], ErrorContext::None)
                        })?;

                        return_value = self
                            .create_value(self.program.return_type(), &return_registers, &mut 0)
                            .map_err(|error| {
                                Error::new(vec![ErrorKind::Vm(error)], ErrorContext::None)
                            })?;

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

    fn create_value(
        &self,
        r#type: &DustType,
        return_registers: &[Register],
        index: &mut usize,
    ) -> Result<Option<DustValue>, VmError> {
        match r#type {
            DustType::Unit if return_registers.is_empty() => Ok(None),
            DustType::Boolean if *index < return_registers.len() => {
                let value = DustValue::Boolean(return_registers[*index].0 != 0);

                *index += 1;

                Ok(Some(value))
            }
            DustType::Character if *index < return_registers.len() => {
                let value = char::from_u32(return_registers[*index].0)
                    .map(DustValue::Character)
                    .ok_or_else(|| VmError::InvalidReturnValue {
                        register_count: return_registers.len(),
                        expected_type: self.program.return_type().clone(),
                    })?;

                *index += 1;

                Ok(Some(value))
            }
            DustType::I8 if *index < return_registers.len() => {
                let value = DustValue::I8(return_registers[*index].0 as i8);

                *index += 1;

                Ok(Some(value))
            }
            DustType::I16 if *index < return_registers.len() => {
                let value = DustValue::I16(return_registers[*index].0 as i16);

                *index += 1;

                Ok(Some(value))
            }
            DustType::I32 if *index < return_registers.len() => {
                let value = DustValue::I32(return_registers[*index].0 as i32);

                *index += 1;

                Ok(Some(value))
            }
            DustType::I64 if *index + 1 < return_registers.len() => {
                let low_bits = return_registers[*index].0 as u64;
                let high_bits = return_registers[*index + 1].0 as u64;
                let value = DustValue::I64(((high_bits << 32) | low_bits) as i64);

                *index += 2;

                Ok(Some(value))
            }
            DustType::I128 if *index + 3 < return_registers.len() => {
                let low_bits = return_registers[*index].0 as u128;
                let mid_low_bits = return_registers[*index + 1].0 as u128;
                let mid_high_bits = return_registers[*index + 2].0 as u128;
                let high_bits = return_registers[*index + 3].0 as u128;
                let value = DustValue::I128(
                    ((high_bits << 96) | (mid_high_bits << 64) | (mid_low_bits << 32) | low_bits)
                        as i128,
                );

                *index += 4;

                Ok(Some(value))
            }
            #[cfg(target_pointer_width = "64")]
            DustType::ISize if *index + 1 < return_registers.len() => {
                let low_bits = return_registers[*index].0 as u64;
                let high_bits = return_registers[*index + 1].0 as u64;
                let value = DustValue::ISize(((high_bits << 32) | low_bits) as isize);

                *index += 2;

                Ok(Some(value))
            }
            #[cfg(target_pointer_width = "32")]
            DustType::ISize if *index < return_registers.len() => {
                let value = DustValue::ISize(return_registers[*index].0 as isize);

                *index += 1;

                Ok(Some(value))
            }
            DustType::U8 if *index < return_registers.len() => {
                let value = DustValue::U8(return_registers[*index].0 as u8);

                *index += 1;

                Ok(Some(value))
            }
            DustType::U16 if *index < return_registers.len() => {
                let value = DustValue::U16(return_registers[*index].0 as u16);

                *index += 1;

                Ok(Some(value))
            }
            DustType::U32 if *index < return_registers.len() => {
                let value = DustValue::U32(return_registers[*index].0);

                *index += 1;

                Ok(Some(value))
            }
            DustType::U64 if *index + 1 < return_registers.len() => {
                let low_bits = return_registers[*index].0 as u64;
                let high_bits = return_registers[*index + 1].0 as u64;
                let value = DustValue::U64((high_bits << 32) | low_bits);

                *index += 2;

                Ok(Some(value))
            }
            DustType::U128 if *index + 3 < return_registers.len() => {
                let low_bits = return_registers[*index].0 as u128;
                let mid_low_bits = return_registers[*index + 1].0 as u128;
                let mid_high_bits = return_registers[*index + 2].0 as u128;
                let high_bits = return_registers[*index + 3].0 as u128;
                let value = DustValue::U128(
                    (high_bits << 96) | (mid_high_bits << 64) | (mid_low_bits << 32) | low_bits,
                );

                *index += 4;

                Ok(Some(value))
            }
            #[cfg(target_pointer_width = "64")]
            DustType::USize if *index + 1 < return_registers.len() => {
                let low_bits = return_registers[*index].0 as u64;
                let high_bits = return_registers[*index + 1].0 as u64;
                let value = DustValue::USize(((high_bits << 32) | low_bits) as usize);

                *index += 2;

                Ok(Some(value))
            }
            #[cfg(target_pointer_width = "32")]
            DustType::USize if *index < return_registers.len() => {
                let value = DustValue::USize(return_registers[*index].0 as usize);

                *index += 1;

                Ok(Some(value))
            }
            DustType::F32 if *index < return_registers.len() => {
                let value = DustValue::F32(f32::from_bits(return_registers[*index].0));

                *index += 1;

                Ok(Some(value))
            }
            DustType::F64 if *index + 1 < return_registers.len() => {
                let low_bits = return_registers[*index].0 as u64;
                let high_bits = return_registers[*index + 1].0 as u64;
                let value = DustValue::F64(f64::from_bits((high_bits << 32) | low_bits));

                *index += 2;

                Ok(Some(value))
            }
            DustType::Tuple(types) => {
                let mut fields = Vec::new();

                for field_type in types {
                    let field_value = self
                        .create_value(field_type, return_registers, index)?
                        .ok_or_else(|| VmError::InvalidReturnValue {
                            register_count: return_registers.len(),
                            expected_type: self.program.return_type().clone(),
                        })?;

                    fields.push(field_value);
                }

                Ok(Some(DustValue::Tuple(fields)))
            }
            DustType::Struct(dust_struct_type) => {
                let DustStructType { name, value_type } = dust_struct_type.as_ref();

                match value_type {
                    DustStructTypeFields::Unit => {
                        Ok(Some(DustValue::Struct(Box::new(DustStruct {
                            struct_name: name.clone(),
                            value: DustStructValue::Unit,
                        }))))
                    }
                    DustStructTypeFields::Tuple(types) => {
                        let mut fields = Vec::new();

                        for field_type in types {
                            let field_value = self
                                .create_value(field_type, return_registers, index)?
                                .ok_or_else(|| VmError::InvalidReturnValue {
                                    register_count: return_registers.len(),
                                    expected_type: self.program.return_type().clone(),
                                })?;

                            fields.push(field_value);
                        }

                        Ok(Some(DustValue::Struct(Box::new(DustStruct {
                            struct_name: name.clone(),
                            value: DustStructValue::Tuple(fields),
                        }))))
                    }
                    DustStructTypeFields::Named(items) => {
                        let mut fields = Vec::new();

                        for (field_name, field_type) in items {
                            let field_value = self
                                .create_value(field_type, return_registers, index)?
                                .ok_or_else(|| VmError::InvalidReturnValue {
                                    register_count: return_registers.len(),
                                    expected_type: self.program.return_type().clone(),
                                })?;

                            fields.push((field_name.clone(), field_value));
                        }

                        Ok(Some(DustValue::Struct(Box::new(DustStruct {
                            struct_name: name.clone(),
                            value: DustStructValue::Struct(fields),
                        }))))
                    }
                }
            }
            DustType::Enum(enum_type) => {
                let DustEnumType { name, variants } = enum_type.as_ref();

                if return_registers.is_empty() {
                    return Err(VmError::InvalidReturnValue {
                        register_count: 0,
                        expected_type: self.program.return_type().clone(),
                    });
                }

                let discriminant = return_registers[*index].0 as usize;

                if discriminant >= variants.len() {
                    return Err(VmError::InvalidReturnValue {
                        register_count: return_registers.len(),
                        expected_type: self.program.return_type().clone(),
                    });
                }

                let (variant_name, variant) = &variants[discriminant];

                match variant {
                    DustStructTypeFields::Unit => {
                        Ok(Some(DustValue::EnumVariant(Box::new(DustEnumVariant {
                            enum_name: name.clone(),
                            variant_name: variant_name.clone(),
                            value: DustStructValue::Unit,
                        }))))
                    }
                    DustStructTypeFields::Tuple(types) => {
                        let mut fields = Vec::new();

                        for field_type in types {
                            let field_value = self
                                .create_value(field_type, return_registers, index)?
                                .ok_or_else(|| VmError::InvalidReturnValue {
                                    register_count: return_registers.len(),
                                    expected_type: self.program.return_type().clone(),
                                })?;

                            fields.push(field_value);
                        }

                        Ok(Some(DustValue::EnumVariant(Box::new(DustEnumVariant {
                            enum_name: name.clone(),
                            variant_name: variant_name.clone(),
                            value: DustStructValue::Tuple(fields),
                        }))))
                    }

                    DustStructTypeFields::Named(items) => {
                        let mut fields = Vec::new();

                        for (field_name, field_type) in items {
                            let field_value = self
                                .create_value(field_type, return_registers, index)?
                                .ok_or_else(|| VmError::InvalidReturnValue {
                                    register_count: return_registers.len(),
                                    expected_type: self.program.return_type().clone(),
                                })?;

                            fields.push((field_name.clone(), field_value));
                        }

                        Ok(Some(DustValue::EnumVariant(Box::new(DustEnumVariant {
                            enum_name: name.clone(),
                            variant_name: variant_name.clone(),
                            value: DustStructValue::Struct(fields),
                        }))))
                    }
                }
            }
            _ => Err(VmError::InvalidReturnValue {
                register_count: return_registers.len(),
                expected_type: self.program.return_type().clone(),
            }),
        }
    }
}
