use cranelift::{
    codegen::{CodegenError, ir::FuncRef},
    frontend::Switch,
    prelude::{
        AbiParam, FunctionBuilder, FunctionBuilderContext, InstBuilder, IntCC, MemFlags, Signature,
        Value as CraneliftValue,
        types::{I8, I32, I64},
    },
};
use cranelift_module::{FuncId, Module, ModuleError};
use tracing::info;

use crate::{
    Address, Chunk, JitCompiler, JitError, MemoryKind, OperandType, Operation, Register,
    instruction::{Add, Call, Jump, Load, NewList, Return, SetList, Subtract},
    jit_vm::call_stack::get_call_frame,
};

pub fn compile_stackless_function(
    compiler: &mut JitCompiler,
    function_id: FuncId,
    chunk: &Chunk,
    is_main: bool,
) -> Result<(), JitError> {
    info!(
        "Compiling stackless function {}",
        chunk.name.as_ref().map_or("anonymous", |path| path.inner())
    );

    let mut function_builder_context = FunctionBuilderContext::new();
    let mut compilation_context = compiler.module.make_context();
    let pointer_type = compiler.module.isa().pointer_type();

    compilation_context
        .func
        .signature
        .params
        .extend([AbiParam::new(pointer_type); 7]);

    let mut function_builder =
        FunctionBuilder::new(&mut compilation_context.func, &mut function_builder_context);

    let allocate_list_function = {
        let mut allocate_list_signature = Signature::new(compiler.module.isa().default_call_conv());

        allocate_list_signature.params.extend([
            AbiParam::new(I8),
            AbiParam::new(I64),
            AbiParam::new(pointer_type),
        ]);
        allocate_list_signature.returns.push(AbiParam::new(I64)); // return value

        compiler.declare_imported_function(
            &mut function_builder,
            "allocate_list",
            allocate_list_signature,
        )?
    };

    let instert_into_list_function = {
        let mut insert_into_list_signature =
            Signature::new(compiler.module.isa().default_call_conv());

        insert_into_list_signature.params.extend([
            AbiParam::new(I64),
            AbiParam::new(I64),
            AbiParam::new(I64),
        ]);
        insert_into_list_signature.returns = vec![];

        compiler.declare_imported_function(
            &mut function_builder,
            "insert_into_list",
            insert_into_list_signature,
        )?
    };

    let allocate_string_function = {
        let mut allocate_string_signature =
            Signature::new(compiler.module.isa().default_call_conv());

        allocate_string_signature.params.extend([
            AbiParam::new(I64),
            AbiParam::new(I64),
            AbiParam::new(pointer_type),
        ]);
        allocate_string_signature.returns.push(AbiParam::new(I64)); // return value

        compiler.declare_imported_function(
            &mut function_builder,
            "allocate_string",
            allocate_string_signature,
        )?
    };

    #[cfg(debug_assertions)]
    let log_operation_function = {
        let mut log_operation_signature = Signature::new(compiler.module.isa().default_call_conv());

        log_operation_signature.params.push(AbiParam::new(I8));
        log_operation_signature.returns = vec![];

        compiler.declare_imported_function(
            &mut function_builder,
            "log_operation",
            log_operation_signature,
        )?
    };

    let bytecode_instructions = &chunk.instructions;
    let instruction_count = bytecode_instructions.len();

    let function_entry_block = function_builder.create_block();
    let mut instruction_blocks = Vec::with_capacity(instruction_count);
    let return_block = function_builder.create_block();
    let mut switch = Switch::new();

    for ip in 0..instruction_count {
        let block = function_builder.create_block();

        instruction_blocks.push(block);
        switch.set_entry(ip as u128, block);
    }

    function_builder.switch_to_block(function_entry_block);
    function_builder.append_block_params_for_function_params(function_entry_block);

    let call_stack_pointer = function_builder.block_params(function_entry_block)[0];
    let call_stack_length_pointer = function_builder.block_params(function_entry_block)[1];
    let register_stack_pointer = function_builder.block_params(function_entry_block)[2];
    let _register_stack_length_pointer = function_builder.block_params(function_entry_block)[3];
    let object_pool_pointer = function_builder.block_params(function_entry_block)[4];
    let return_register_pointer = function_builder.block_params(function_entry_block)[5];
    let return_type_pointer = function_builder.block_params(function_entry_block)[6];

    let call_stack_length =
        function_builder
            .ins()
            .load(I64, MemFlags::new(), call_stack_length_pointer, 0);
    let one = function_builder.ins().iconst(I64, 1);
    let top_call_frame_index = function_builder.ins().isub(call_stack_length, one);

    let (
        current_frame_ip,
        _current_frame_function_index,
        current_frame_register_range_start,
        _current_frame_register_range_end,
        _current_frame_arguments_index,
        current_frame_destination_index,
    ) = get_call_frame(
        top_call_frame_index,
        call_stack_pointer,
        &mut function_builder,
    );

    let current_frame_register_base_offset = function_builder.ins().imul_imm(
        current_frame_register_range_start,
        size_of::<Register>() as i64,
    );
    let current_frame_base_address = function_builder
        .ins()
        .iadd(register_stack_pointer, current_frame_register_base_offset);

    switch.emit(&mut function_builder, current_frame_ip, return_block);

    for ip in 0..instruction_count {
        let current_instruction = &bytecode_instructions[ip];
        let operation = current_instruction.operation();
        let instruction_block = instruction_blocks[ip];

        function_builder.switch_to_block(instruction_block);

        info!("Compiling {operation} at IP {ip}");

        #[cfg(debug_assertions)]
        {
            use cranelift::prelude::types::I8;

            let operation_code_instruction = function_builder.ins().iconst(I8, operation.0 as i64);

            function_builder
                .ins()
                .call(log_operation_function, &[operation_code_instruction]);
        }

        match operation {
            Operation::LOAD => {
                let Load {
                    destination,
                    operand,
                    r#type,
                    jump_next,
                } = Load::from(*current_instruction);
                let result_register = match r#type {
                    OperandType::BOOLEAN => {
                        get_boolean(operand, current_frame_base_address, &mut function_builder)?
                    }
                    OperandType::BYTE => {
                        get_byte(operand, current_frame_base_address, &mut function_builder)?
                    }
                    OperandType::CHARACTER => get_character(
                        operand,
                        current_frame_base_address,
                        chunk,
                        &mut function_builder,
                    )?,
                    OperandType::FLOAT => get_float(
                        operand,
                        current_frame_base_address,
                        chunk,
                        &mut function_builder,
                    )?,
                    OperandType::INTEGER => get_integer(
                        operand,
                        current_frame_base_address,
                        chunk,
                        &mut function_builder,
                    )?,
                    OperandType::STRING => get_string(
                        operand,
                        current_frame_base_address,
                        chunk,
                        object_pool_pointer,
                        allocate_string_function,
                        &mut function_builder,
                    )?,
                    OperandType::LIST_INTEGER => {
                        get_list(operand, current_frame_base_address, &mut function_builder)?
                    }
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                compiler.set_register(
                    destination.index as usize,
                    result_register,
                    current_frame_base_address,
                    &mut function_builder,
                )?;

                if jump_next {
                    compiler.emit_jump(ip, 2, &mut function_builder, &[])?;
                }
            }
            Operation::NEW_LIST => {
                let NewList {
                    destination,
                    length,
                    list_type,
                } = NewList::from(*current_instruction);
                let list_type = function_builder.ins().iconst(I8, list_type.0 as i64);
                let list_length = function_builder.ins().iconst(I64, length as i64);
                let call_allocate_list_instruction = function_builder.ins().call(
                    allocate_list_function,
                    &[list_type, list_length, object_pool_pointer],
                );
                let list_object_pointer =
                    function_builder.inst_results(call_allocate_list_instruction)[0];

                compiler.set_register(
                    destination.index as usize,
                    list_object_pointer,
                    current_frame_base_address,
                    &mut function_builder,
                )?;
            }
            Operation::SET_LIST => {
                let SetList {
                    destination_list,
                    item_source,
                    list_index,
                    item_type,
                } = SetList::from(*current_instruction);
                let list_pointer = get_list(
                    destination_list,
                    current_frame_base_address,
                    &mut function_builder,
                )?;
                let item_value = match item_type {
                    OperandType::INTEGER => get_integer(
                        item_source,
                        current_frame_base_address,
                        chunk,
                        &mut function_builder,
                    )?,
                    OperandType::BOOLEAN => get_boolean(
                        item_source,
                        current_frame_base_address,
                        &mut function_builder,
                    )?,
                    OperandType::BYTE => get_byte(
                        item_source,
                        current_frame_base_address,
                        &mut function_builder,
                    )?,
                    OperandType::CHARACTER => get_character(
                        item_source,
                        current_frame_base_address,
                        chunk,
                        &mut function_builder,
                    )?,
                    OperandType::FLOAT => get_float(
                        item_source,
                        current_frame_base_address,
                        chunk,
                        &mut function_builder,
                    )?,
                    OperandType::STRING => get_string(
                        item_source,
                        current_frame_base_address,
                        chunk,
                        object_pool_pointer,
                        allocate_string_function,
                        &mut function_builder,
                    )?,
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: item_type,
                        });
                    }
                };
                let list_index = function_builder.ins().iconst(I64, list_index as i64);

                function_builder.ins().call(
                    instert_into_list_function,
                    &[list_pointer, list_index, item_value],
                );
            }
            Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL => {
                let comparator = current_instruction.a_field();
                let left = current_instruction.b_address();
                let right = current_instruction.c_address();
                let r#type = current_instruction.operand_type();
                let operation = current_instruction.operation();
                let comparison = match (operation, comparator != 0) {
                    (Operation::EQUAL, true) => IntCC::Equal,
                    (Operation::EQUAL, false) => IntCC::NotEqual,
                    (Operation::LESS, true) => IntCC::SignedLessThan,
                    (Operation::LESS, false) => IntCC::SignedGreaterThanOrEqual,
                    (Operation::LESS_EQUAL, true) => IntCC::SignedLessThanOrEqual,
                    (Operation::LESS_EQUAL, false) => IntCC::SignedGreaterThan,
                    _ => unreachable!(),
                };
                let comparison_result = match r#type {
                    OperandType::INTEGER => {
                        let left_value = get_integer(
                            left,
                            current_frame_base_address,
                            chunk,
                            &mut function_builder,
                        )?;
                        let right_value = get_integer(
                            right,
                            current_frame_base_address,
                            chunk,
                            &mut function_builder,
                        )?;

                        function_builder
                            .ins()
                            .icmp(comparison, left_value, right_value)
                    }
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                function_builder.ins().brif(
                    comparison_result,
                    instruction_blocks[ip + 2],
                    &[],
                    instruction_blocks[ip + 1],
                    &[],
                );
            }
            Operation::ADD => {
                let Add {
                    destination,
                    left,
                    right,
                    r#type,
                } = Add::from(*current_instruction);
                let result_register = match r#type {
                    OperandType::INTEGER => {
                        let left_value = get_integer(
                            left,
                            current_frame_base_address,
                            chunk,
                            &mut function_builder,
                        )?;
                        let right_value = get_integer(
                            right,
                            current_frame_base_address,
                            chunk,
                            &mut function_builder,
                        )?;

                        function_builder.ins().iadd(left_value, right_value)
                    }
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                compiler.set_register(
                    destination.index as usize,
                    result_register,
                    current_frame_base_address,
                    &mut function_builder,
                )?;
            }
            Operation::SUBTRACT => {
                let Subtract {
                    destination,
                    left,
                    right,
                    r#type,
                } = Subtract::from(*current_instruction);
                let result_register = match r#type {
                    OperandType::INTEGER => {
                        let left_value = get_integer(
                            left,
                            current_frame_base_address,
                            chunk,
                            &mut function_builder,
                        )?;
                        let right_value = get_integer(
                            right,
                            current_frame_base_address,
                            chunk,
                            &mut function_builder,
                        )?;

                        function_builder.ins().isub(left_value, right_value)
                    }
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                compiler.set_register(
                    destination.index as usize,
                    result_register,
                    current_frame_base_address,
                    &mut function_builder,
                )?;
            }
            Operation::CALL => {
                let Call {
                    destination,
                    prototype_index,
                    arguments_index,
                    return_type: _,
                } = Call::from(*current_instruction);
                let destination_index = destination.index as usize;
                let prototype_index = prototype_index as usize;
                let arguments_index = arguments_index as usize;
                let callee_function_ids = compiler.function_ids.get(prototype_index).ok_or(
                    JitError::FunctionIndexOutOfBounds {
                        ip,
                        function_index: prototype_index,
                        total_function_count: compiler.function_ids.len(),
                    },
                )?;
                let callee_function_reference = compiler
                    .module
                    .declare_func_in_func(callee_function_ids.direct, function_builder.func);

                let call_arguments_list = chunk.call_argument_lists.get(arguments_index).ok_or(
                    JitError::ArgumentsIndexOutOfBounds {
                        arguments_index,
                        total_argument_count: chunk.call_argument_lists.len(),
                    },
                )?;
                let mut arguments = Vec::with_capacity(call_arguments_list.len() + 3);

                for (address, r#type) in call_arguments_list {
                    let argument_value = match *r#type {
                        OperandType::INTEGER => {
                            let integer_value = get_integer(
                                *address,
                                current_frame_base_address,
                                chunk,
                                &mut function_builder,
                            )?;

                            Ok(integer_value)
                        }?,
                        _ => {
                            return Err(JitError::UnsupportedOperandType {
                                operand_type: *r#type,
                            });
                        }
                    };

                    arguments.push(argument_value);
                }

                let call_instruction = function_builder
                    .ins()
                    .call(callee_function_reference, &arguments);
                let return_value = function_builder.inst_results(call_instruction)[0];

                compiler.set_register(
                    destination_index,
                    return_value,
                    current_frame_base_address,
                    &mut function_builder,
                )?;

                function_builder.ins().jump(instruction_blocks[ip + 1], &[]);
            }
            Operation::JUMP => {
                let Jump {
                    offset,
                    is_positive,
                } = Jump::from(*current_instruction);

                if is_positive {
                    compiler.emit_jump(
                        ip,
                        (offset + 1) as isize,
                        &mut function_builder,
                        &instruction_blocks,
                    )?;
                } else {
                    compiler.emit_jump(
                        ip,
                        -(offset as isize),
                        &mut function_builder,
                        &instruction_blocks,
                    )?;
                }
            }
            Operation::RETURN => {
                let Return {
                    should_return_value,
                    return_value_address,
                    r#type,
                } = Return::from(*current_instruction);

                if should_return_value {
                    let (value_to_return, return_type) = match r#type {
                        OperandType::BOOLEAN => {
                            let boolean_value = get_boolean(
                                return_value_address,
                                current_frame_base_address,
                                &mut function_builder,
                            )?;
                            let boolean_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::BOOLEAN.0 as i64);

                            (boolean_value, boolean_type)
                        }
                        OperandType::BYTE => {
                            let byte_value = get_byte(
                                return_value_address,
                                current_frame_base_address,
                                &mut function_builder,
                            )?;
                            let byte_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::BYTE.0 as i64);

                            (byte_value, byte_type)
                        }
                        OperandType::CHARACTER => {
                            let character_value = get_character(
                                return_value_address,
                                current_frame_base_address,
                                chunk,
                                &mut function_builder,
                            )?;
                            let character_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::CHARACTER.0 as i64);

                            (character_value, character_type)
                        }
                        OperandType::FLOAT => {
                            let float_value = get_float(
                                return_value_address,
                                current_frame_base_address,
                                chunk,
                                &mut function_builder,
                            )?;
                            let float_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::FLOAT.0 as i64);

                            (float_value, float_type)
                        }
                        OperandType::INTEGER => {
                            let integer_value = get_integer(
                                return_value_address,
                                current_frame_base_address,
                                chunk,
                                &mut function_builder,
                            )?;
                            let integer_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::INTEGER.0 as i64);

                            (integer_value, integer_type)
                        }
                        OperandType::STRING => {
                            let string_value = get_string(
                                return_value_address,
                                current_frame_base_address,
                                chunk,
                                object_pool_pointer,
                                allocate_string_function,
                                &mut function_builder,
                            )?;
                            let string_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::STRING.0 as i64);

                            (string_value, string_type)
                        }
                        OperandType::LIST_INTEGER => {
                            let list_value = get_list(
                                return_value_address,
                                current_frame_base_address,
                                &mut function_builder,
                            )?;
                            let list_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::LIST_INTEGER.0 as i64);

                            (list_value, list_type)
                        }
                        _ => {
                            return Err(JitError::UnsupportedOperandType {
                                operand_type: r#type,
                            });
                        }
                    };

                    if is_main {
                        function_builder.ins().store(
                            MemFlags::new(),
                            value_to_return,
                            return_register_pointer,
                            0,
                        );
                        function_builder.ins().store(
                            MemFlags::new(),
                            return_type,
                            return_type_pointer,
                            0,
                        );
                    } else {
                        let byte_offset = function_builder.ins().imul_imm(
                            current_frame_destination_index,
                            size_of::<Register>() as i64,
                        );
                        let destination_address = function_builder
                            .ins()
                            .iadd(register_stack_pointer, byte_offset);

                        function_builder.ins().store(
                            MemFlags::new(),
                            value_to_return,
                            destination_address,
                            0,
                        );
                    }
                }

                let current_length =
                    function_builder
                        .ins()
                        .load(I64, MemFlags::new(), call_stack_length_pointer, 0);
                let new_length = function_builder.ins().isub(current_length, one);

                function_builder.ins().store(
                    MemFlags::new(),
                    new_length,
                    call_stack_length_pointer,
                    0,
                );

                function_builder.ins().return_(&[]);
            }

            _ => {
                return Err(JitError::UnhandledOperation { operation });
            }
        }

        if !matches!(
            operation,
            Operation::EQUAL
                | Operation::LESS
                | Operation::LESS_EQUAL
                | Operation::CALL
                | Operation::JUMP
                | Operation::RETURN
        ) {
            compiler.emit_jump(ip, 1, &mut function_builder, &instruction_blocks)?;
        }
    }

    function_builder.switch_to_block(return_block);
    function_builder.ins().return_(&[]);
    function_builder.seal_all_blocks();

    compiler
        .module
        .define_function(function_id, &mut compilation_context)
        .map_err(|error| {
            if let ModuleError::Compilation(CodegenError::Verifier(errors)) = error {
                let message = errors
                    .0
                    .iter()
                    .map(|error| format!("\n{error}"))
                    .collect::<String>();

                JitError::CraneliftModuleError { message }
            } else {
                JitError::CraneliftModuleError {
                    message: error.to_string(),
                }
            }
        })?;

    info!(
        "Finished compiling stackless function {}",
        chunk.name.as_ref().map_or("anonymous", |path| path.inner()),
    );

    compiler.module.clear_context(&mut compilation_context);

    Ok(())
}

fn get_boolean(
    address: Address,
    frame_base_address: CraneliftValue,
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    let jit_value = match address.memory {
        MemoryKind::REGISTER => {
            let relative_index = function_builder.ins().iconst(I64, address.index as i64);
            let byte_offset = function_builder
                .ins()
                .imul_imm(relative_index, size_of::<Register>() as i64);
            let address = function_builder.ins().iadd(frame_base_address, byte_offset);

            function_builder.ins().load(I8, MemFlags::new(), address, 0)
        }
        MemoryKind::ENCODED => {
            let boolean_value = address.index != 0;

            function_builder.ins().iconst(I8, boolean_value as i64)
        }
        _ => {
            return Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            });
        }
    };

    Ok(jit_value)
}

fn get_byte(
    address: Address,
    frame_base_address: CraneliftValue,
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    let jit_value = match address.memory {
        MemoryKind::REGISTER => {
            let relative_index = function_builder.ins().iconst(I64, address.index as i64);
            let byte_offset = function_builder
                .ins()
                .imul_imm(relative_index, size_of::<Register>() as i64);
            let address = function_builder.ins().iadd(frame_base_address, byte_offset);

            function_builder.ins().load(I8, MemFlags::new(), address, 0)
        }
        MemoryKind::ENCODED => function_builder.ins().iconst(I8, address.index as i64),
        _ => {
            return Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            });
        }
    };

    Ok(jit_value)
}

fn get_character(
    address: Address,
    frame_base_address: CraneliftValue,
    chunk: &Chunk,
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    let jit_value = match address.memory {
        MemoryKind::REGISTER => {
            let relative_index = function_builder.ins().iconst(I64, address.index as i64);
            let byte_offset = function_builder
                .ins()
                .imul_imm(relative_index, size_of::<Register>() as i64);
            let address = function_builder.ins().iadd(frame_base_address, byte_offset);

            function_builder
                .ins()
                .load(I32, MemFlags::new(), address, 0)
        }
        MemoryKind::CONSTANT => {
            let constant = chunk.constants.get(address.index as usize).ok_or(
                JitError::ConstantIndexOutOfBounds {
                    constant_index: address.index as usize,
                    total_constant_count: chunk.constants.len(),
                },
            )?;
            let character = match constant.as_character() {
                Some(character) => character,
                None => {
                    return Err(JitError::InvalidConstantType {
                        expected_type: OperandType::CHARACTER,
                    });
                }
            };

            function_builder.ins().iconst(I32, character as i64)
        }
        _ => {
            return Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            });
        }
    };

    Ok(jit_value)
}

fn get_float(
    address: Address,
    frame_base_address: CraneliftValue,
    chunk: &Chunk,
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    let address_index = address.index as usize;
    let jit_value =
        match address.memory {
            MemoryKind::REGISTER => {
                let relative_index = function_builder.ins().iconst(I64, address.index as i64);
                let byte_offset = function_builder
                    .ins()
                    .imul_imm(relative_index, size_of::<Register>() as i64);
                let address = function_builder.ins().iadd(frame_base_address, byte_offset);

                function_builder
                    .ins()
                    .load(I64, MemFlags::new(), address, 0)
            }
            MemoryKind::CONSTANT => {
                let constant = chunk.constants.get(address_index).ok_or(
                    JitError::ConstantIndexOutOfBounds {
                        constant_index: address_index,
                        total_constant_count: chunk.constants.len(),
                    },
                )?;
                let float = match constant.as_float() {
                    Some(float_value) => float_value,
                    None => {
                        return Err(JitError::InvalidConstantType {
                            expected_type: OperandType::FLOAT,
                        });
                    }
                };

                function_builder.ins().iconst(I64, float.to_bits() as i64)
            }
            _ => {
                return Err(JitError::UnsupportedMemoryKind {
                    memory_kind: address.memory,
                });
            }
        };

    Ok(jit_value)
}

fn get_integer(
    address: Address,
    frame_base_address: CraneliftValue,
    chunk: &Chunk,
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    let address_index = address.index as usize;
    let jit_value =
        match address.memory {
            MemoryKind::REGISTER => {
                let relative_index = function_builder.ins().iconst(I64, address.index as i64);
                let byte_offset = function_builder
                    .ins()
                    .imul_imm(relative_index, size_of::<Register>() as i64);
                let address = function_builder.ins().iadd(frame_base_address, byte_offset);

                function_builder
                    .ins()
                    .load(I64, MemFlags::new(), address, 0)
            }
            MemoryKind::CONSTANT => {
                let constant = chunk.constants.get(address_index).ok_or(
                    JitError::ConstantIndexOutOfBounds {
                        constant_index: address_index,
                        total_constant_count: chunk.constants.len(),
                    },
                )?;
                let integer = match constant.as_integer() {
                    Some(integer) => integer,
                    None => {
                        return Err(JitError::InvalidConstantType {
                            expected_type: OperandType::INTEGER,
                        });
                    }
                };

                function_builder.ins().iconst(I64, integer)
            }
            _ => {
                return Err(JitError::UnsupportedMemoryKind {
                    memory_kind: address.memory,
                });
            }
        };

    Ok(jit_value)
}

fn get_string(
    address: Address,
    frame_base_address: CraneliftValue,
    chunk: &Chunk,
    object_pool_pointer: CraneliftValue,
    allocate_string_function: FuncRef,
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    let address_index = address.index as usize;
    let jit_value =
        match address.memory {
            MemoryKind::REGISTER => {
                let relative_index = function_builder.ins().iconst(I64, address.index as i64);
                let byte_offset = function_builder
                    .ins()
                    .imul_imm(relative_index, size_of::<Register>() as i64);
                let address = function_builder.ins().iadd(frame_base_address, byte_offset);

                function_builder
                    .ins()
                    .load(I64, MemFlags::new(), address, 0)
            }
            MemoryKind::CONSTANT => {
                let constant = chunk.constants.get(address_index).ok_or(
                    JitError::ConstantIndexOutOfBounds {
                        constant_index: address_index,
                        total_constant_count: chunk.constants.len(),
                    },
                )?;
                let string = match constant.as_string() {
                    Some(string) => string,
                    None => {
                        return Err(JitError::InvalidConstantType {
                            expected_type: OperandType::STRING,
                        });
                    }
                };
                let string_pointer = function_builder
                    .ins()
                    .iconst(I64, string.as_ptr() as usize as i64);
                let string_length = function_builder.ins().iconst(I64, string.len() as i64);
                let call_allocate_string_instruction = function_builder.ins().call(
                    allocate_string_function,
                    &[string_pointer, string_length, object_pool_pointer],
                );

                function_builder.inst_results(call_allocate_string_instruction)[0]
            }
            _ => {
                return Err(JitError::UnsupportedMemoryKind {
                    memory_kind: address.memory,
                });
            }
        };

    Ok(jit_value)
}

fn get_list(
    address: Address,
    frame_base_address: CraneliftValue,
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    let jit_value = match address.memory {
        MemoryKind::REGISTER => {
            let relative_index = function_builder.ins().iconst(I64, address.index as i64);
            let byte_offset = function_builder
                .ins()
                .imul_imm(relative_index, size_of::<Register>() as i64);
            let address = function_builder.ins().iadd(frame_base_address, byte_offset);

            function_builder
                .ins()
                .load(I64, MemFlags::new(), address, 0)
        }
        _ => {
            return Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            });
        }
    };

    Ok(jit_value)
}
