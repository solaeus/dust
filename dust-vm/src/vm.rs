use std::sync::Arc;

use dust_compiler::{
    dust_type::{DustEnumType, DustStructType, DustStructTypeFields, DustType},
    dust_value::{DustEnumVariant, DustStruct, DustStructValue, DustValue},
    program::Program,
};
use tracing::{Level, error, info, span};

use crate::{
    error::VmError,
    register::Register,
    thread_pool::{ThreadMessage, ThreadPool},
};

pub struct Vm {
    program: Arc<Program>,
    thread_pool: ThreadPool,
}

impl Vm {
    pub fn new(program: Program, config: VmConfig) -> Self {
        let program = Arc::new(program);

        Self {
            program: Arc::clone(&program),
            thread_pool: ThreadPool::new(
                program,
                config.minimum_object_heap,
                config.minimum_object_sweep,
            ),
        }
    }

    pub fn run(self) -> Result<Option<DustValue>, VmError> {
        let span = span!(Level::INFO, "vm");
        let _enter = span.enter();

        let message_receiver = {
            info!("Spawning main VM thread");

            let mut spawner = self.thread_pool.lock_spawner();

            spawner.spawn_thread(0)?;
            spawner.clone_message_receiver()
        };

        let mut return_value: Option<DustValue> = None;

        loop {
            match message_receiver.recv() {
                Ok(ThreadMessage::SpawnThread {
                    thread_name,
                    prototype_index,
                }) => {
                    info!("Spawning VM thread: {thread_name} with proto_{prototype_index}");

                    self.thread_pool
                        .lock_spawner()
                        .spawn_named_thread(thread_name, prototype_index)?;
                }
                Ok(ThreadMessage::ThreadFinished {
                    thread_id,
                    return_registers,
                }) => {
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

                        let return_type = self.program.return_type();

                        if return_type != &DustType::Unit {
                            return_value = Some(self.create_return_value(
                                return_type,
                                &return_registers,
                                &mut 0,
                            )?);
                        }

                        break;
                    }
                }
                Ok(ThreadMessage::ThreadError { thread_id, error }) => {
                    error!(
                        "VM thread encountered an error: Thread ID: {}, Error: {}",
                        thread_id.as_u64(),
                        error
                    );

                    return Err(error);
                }
                Err(error) => {
                    error!("VM main thread encountered an error: {}", error);

                    return Err(VmError::ChannelError);
                }
            }
        }

        Ok(return_value)
    }

    fn create_return_value(
        &self,
        r#type: &DustType,
        return_registers: &[Register],
        index: &mut usize,
    ) -> Result<DustValue, VmError> {
        match r#type {
            DustType::Unit if return_registers.is_empty() => Ok(DustValue::Tuple(Vec::new())),
            DustType::Boolean if *index < return_registers.len() => {
                let boolean = return_registers[*index].as_value();
                *index += 1;

                Ok(DustValue::Boolean(boolean))
            }
            DustType::Character if *index < return_registers.len() => {
                let character = return_registers[*index].as_character()?;
                *index += 1;

                Ok(DustValue::Character(character))
            }
            DustType::I8 if *index < return_registers.len() => {
                let integer = return_registers[*index].as_value();
                *index += 1;

                Ok(DustValue::I8(integer))
            }
            DustType::I16 if *index < return_registers.len() => {
                let integer = return_registers[*index].as_value();
                *index += 1;

                Ok(DustValue::I16(integer))
            }
            DustType::I32 if *index < return_registers.len() => {
                let integer = return_registers[*index].as_value();
                *index += 1;

                Ok(DustValue::I32(integer))
            }
            DustType::I64 if *index + 1 < return_registers.len() => {
                let low_bits = return_registers[*index].as_bits() as u64;
                let high_bits = return_registers[*index + 1].as_bits() as u64;
                *index += 2;

                Ok(DustValue::I64(((high_bits << 32) | low_bits) as i64))
            }
            DustType::I128 if *index + 3 < return_registers.len() => {
                let low_bits = return_registers[*index].as_bits() as u128;
                let mid_low_bits = return_registers[*index + 1].as_bits() as u128;
                let mid_high_bits = return_registers[*index + 2].as_bits() as u128;
                let high_bits = return_registers[*index + 3].as_bits() as u128;
                *index += 4;

                Ok(DustValue::I128(
                    ((high_bits << 96) | (mid_high_bits << 64) | (mid_low_bits << 32) | low_bits)
                        as i128,
                ))
            }
            #[cfg(target_pointer_width = "64")]
            DustType::ISize if *index + 1 < return_registers.len() => {
                let low_bits = return_registers[*index].as_bits() as u64;
                let high_bits = return_registers[*index + 1].as_bits() as u64;
                *index += 2;

                Ok(DustValue::ISize(((high_bits << 32) | low_bits) as isize))
            }
            #[cfg(target_pointer_width = "32")]
            DustType::ISize if *index < return_registers.len() => {
                let integer = return_registers[*index].get::<i32>() as isize;
                *index += 1;

                Ok(DustValue::ISize(integer))
            }
            DustType::U8 if *index < return_registers.len() => {
                let integer = return_registers[*index].as_value();
                *index += 1;

                Ok(DustValue::U8(integer))
            }
            DustType::U16 if *index < return_registers.len() => {
                let value = return_registers[*index].as_value();
                *index += 1;

                Ok(DustValue::U16(value))
            }
            DustType::U32 if *index < return_registers.len() => {
                let value = return_registers[*index].as_value();
                *index += 1;

                Ok(DustValue::U32(value))
            }
            DustType::U64 if *index + 1 < return_registers.len() => {
                let low_bits = return_registers[*index].as_bits() as u64;
                let high_bits = return_registers[*index + 1].as_bits() as u64;
                *index += 2;

                Ok(DustValue::U64(high_bits << 32 | low_bits))
            }
            DustType::U128 if *index + 3 < return_registers.len() => {
                let low_bits = return_registers[*index].as_bits() as u128;
                let mid_low_bits = return_registers[*index + 1].as_bits() as u128;
                let mid_high_bits = return_registers[*index + 2].as_bits() as u128;
                let high_bits = return_registers[*index + 3].as_bits() as u128;
                *index += 4;

                Ok(DustValue::U128(
                    high_bits << 96 | mid_high_bits << 64 | mid_low_bits << 32 | low_bits,
                ))
            }
            #[cfg(target_pointer_width = "64")]
            DustType::USize if *index + 1 < return_registers.len() => {
                let low_bits = return_registers[*index].as_bits() as u64;
                let high_bits = return_registers[*index + 1].as_bits() as u64;
                let value = DustValue::USize(((high_bits << 32) | low_bits) as usize);
                *index += 2;

                Ok(value)
            }
            #[cfg(target_pointer_width = "32")]
            DustType::USize if *index < return_registers.len() => {
                let value = DustValue::USize(return_registers[*index].0 as usize);
                *index += 1;

                Ok(value)
            }
            DustType::F32 if *index < return_registers.len() => {
                let float = return_registers[*index].as_value();
                *index += 1;

                Ok(DustValue::F32(float))
            }
            DustType::F64 if *index + 1 < return_registers.len() => {
                let low_bits = return_registers[*index].as_bits() as u64;
                let high_bits = return_registers[*index + 1].as_bits() as u64;
                *index += 2;

                Ok(DustValue::F64(f64::from_bits(high_bits << 32 | low_bits)))
            }
            DustType::Tuple(types) => {
                let mut fields = Vec::new();

                for field_type in types {
                    let field_value =
                        self.create_return_value(field_type, return_registers, index)?;

                    fields.push(field_value);
                }

                Ok(DustValue::Tuple(fields))
            }
            DustType::Struct(dust_struct_type) => {
                let DustStructType { name, value_type } = dust_struct_type.as_ref();

                match value_type {
                    DustStructTypeFields::Unit => Ok(DustValue::Struct(Box::new(DustStruct {
                        struct_name: name.clone(),
                        value: DustStructValue::Unit,
                    }))),
                    DustStructTypeFields::Tuple(types) => {
                        let mut fields = Vec::new();

                        for field_type in types {
                            let field_value =
                                self.create_return_value(field_type, return_registers, index)?;

                            fields.push(field_value);
                        }

                        Ok(DustValue::Struct(Box::new(DustStruct {
                            struct_name: name.clone(),
                            value: DustStructValue::Tuple(fields),
                        })))
                    }
                    DustStructTypeFields::Named(items) => {
                        let mut fields = Vec::new();

                        for (field_name, field_type) in items {
                            let field_value =
                                self.create_return_value(field_type, return_registers, index)?;

                            fields.push((field_name.clone(), field_value));
                        }

                        Ok(DustValue::Struct(Box::new(DustStruct {
                            struct_name: name.clone(),
                            value: DustStructValue::Struct(fields),
                        })))
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

                let discriminant = return_registers[*index].as_value::<u16>() as usize;

                if discriminant >= variants.len() {
                    return Err(VmError::InvalidReturnValue {
                        register_count: return_registers.len(),
                        expected_type: self.program.return_type().clone(),
                    });
                }

                let (variant_name, variant) = &variants[discriminant];

                match variant {
                    DustStructTypeFields::Unit => {
                        Ok(DustValue::EnumVariant(Box::new(DustEnumVariant {
                            enum_name: name.clone(),
                            variant_name: variant_name.clone(),
                            value: DustStructValue::Unit,
                        })))
                    }
                    DustStructTypeFields::Tuple(types) => {
                        let mut fields = Vec::new();

                        for field_type in types {
                            let field_value =
                                self.create_return_value(field_type, return_registers, index)?;

                            fields.push(field_value);
                        }

                        Ok(DustValue::EnumVariant(Box::new(DustEnumVariant {
                            enum_name: name.clone(),
                            variant_name: variant_name.clone(),
                            value: DustStructValue::Tuple(fields),
                        })))
                    }

                    DustStructTypeFields::Named(items) => {
                        let mut fields = Vec::new();

                        for (field_name, field_type) in items {
                            let field_value =
                                self.create_return_value(field_type, return_registers, index)?;

                            fields.push((field_name.clone(), field_value));
                        }

                        Ok(DustValue::EnumVariant(Box::new(DustEnumVariant {
                            enum_name: name.clone(),
                            variant_name: variant_name.clone(),
                            value: DustStructValue::Struct(fields),
                        })))
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

#[derive(Default)]
pub struct VmConfig {
    pub minimum_object_heap: usize,
    pub minimum_object_sweep: usize,
}
