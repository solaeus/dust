use cranelift::{
    codegen::{CodegenError, ir::FuncRef},
    prelude::{
        AbiParam, FunctionBuilder, FunctionBuilderContext, InstBuilder, IntCC, Signature,
        Value as CraneliftValue, types::I64,
    },
};
use cranelift_module::{FuncId, Module, ModuleError};
use tracing::{Level, info, span};

use crate::{
    chunk::Chunk,
    constant_table::ConstantTable,
    instruction::{
        Add, Address, Call, CallNative, Jump, Load, MemoryKind, OperandType, Operation, Return,
    },
    jit_vm::{JitCompiler, JitError, jit_compiler::FunctionIds},
    r#type::Type,
};

pub fn compile_direct_function(
    compiler: &mut JitCompiler,
    function_id: FuncId,
    chunk: &Chunk,
) -> Result<(), JitError> {
    let span = span!(Level::INFO, "direct");
    let _enter = span.enter();

    let mut function_builder_context = FunctionBuilderContext::new();
    let mut compilation_context = compiler.module.make_context();
    let pointer_type = compiler.module.target_config().pointer_type();
    let value_parameter_count = chunk.r#type.value_parameters.len();

    for _ in 0..value_parameter_count {
        compilation_context
            .func
            .signature
            .params
            .push(AbiParam::new(I64));
    }

    compilation_context
        .func
        .signature
        .params
        .push(AbiParam::new(pointer_type));
    compilation_context
        .func
        .signature
        .returns
        .push(AbiParam::new(I64));

    let mut function_builder =
        FunctionBuilder::new(&mut compilation_context.func, &mut function_builder_context);

    #[cfg(debug_assertions)]
    let log_operation_and_ip_function = {
        use cranelift::prelude::{Signature, types::I8};

        let mut log_operation_signature = Signature::new(compiler.module.isa().default_call_conv());

        log_operation_signature.params.push(AbiParam::new(I8));
        log_operation_signature.params.push(AbiParam::new(I64));
        log_operation_signature.returns = vec![];

        compiler.declare_imported_function(
            &mut function_builder,
            "log_operation_and_ip",
            log_operation_signature,
        )?
    };

    let allocate_string_function = {
        let mut allocate_string_signature =
            Signature::new(compiler.module.isa().default_call_conv());

        allocate_string_signature.params.extend([
            AbiParam::new(pointer_type),
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

    let read_line_function = {
        let mut read_line_signature = Signature::new(compiler.module.isa().default_call_conv());

        read_line_signature.params.push(AbiParam::new(pointer_type));
        read_line_signature.returns.push(AbiParam::new(I64));

        compiler.declare_imported_function(
            &mut function_builder,
            "read_line",
            read_line_signature,
        )?
    };

    let write_line_function = {
        let mut write_line_signature = Signature::new(compiler.module.isa().default_call_conv());

        write_line_signature
            .params
            .extend([AbiParam::new(pointer_type), AbiParam::new(I64)]);
        write_line_signature.returns = vec![];

        compiler.declare_imported_function(
            &mut function_builder,
            "write_line",
            write_line_signature,
        )?
    };

    let bytecode_instructions = &chunk.instructions;
    let instruction_count = bytecode_instructions.len();

    let function_entry_block = function_builder.create_block();
    let mut instruction_blocks = Vec::with_capacity(instruction_count);
    let return_block = function_builder.create_block();

    for _ in 0..instruction_count {
        let block = function_builder.create_block();

        instruction_blocks.push(block);
    }

    function_builder.switch_to_block(function_entry_block);
    function_builder.append_block_params_for_function_params(function_entry_block);

    let entry_block_params = function_builder.block_params(function_entry_block);
    let thread_context = *entry_block_params.last().unwrap();

    let function_arguments = entry_block_params[..entry_block_params.len() - 1].to_vec();
    let mut ssa_registers = vec![CraneliftValue::from_u32(0); chunk.register_count as usize];

    for (index, argument) in function_arguments.iter().enumerate() {
        ssa_registers[index] = *argument;
    }

    function_builder.ins().jump(instruction_blocks[0], &[]);

    for ip in 0..instruction_count {
        let current_instruction = &bytecode_instructions[ip];
        let operation = current_instruction.operation();
        let instruction_block = instruction_blocks[ip];

        function_builder.switch_to_block(instruction_block);

        info!("Compiling {operation} at IP {ip} in direct function");

        #[cfg(debug_assertions)]
        {
            use cranelift::prelude::types::I8;

            let operation_code_instruction = function_builder.ins().iconst(I8, operation.0 as i64);
            let ip_instruction = function_builder.ins().iconst(I64, ip as i64);

            function_builder.ins().call(
                log_operation_and_ip_function,
                &[operation_code_instruction, ip_instruction],
            );
        }

        match operation {
            Operation::LOAD => {
                let Load {
                    destination,
                    operand,
                    r#type,
                    jump_next,
                } = Load::from(*current_instruction);
                let destination_index = destination.index as usize;
                let value = match r#type {
                    OperandType::INTEGER => self::get_integer(
                        operand,
                        &compiler.program.constants,
                        &ssa_registers,
                        &mut function_builder,
                    )?,
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                ssa_registers[destination_index] = value;

                if jump_next {
                    function_builder.ins().jump(instruction_blocks[ip + 2], &[]);
                } else {
                    function_builder.ins().jump(instruction_blocks[ip + 1], &[]);
                }

                Ok(())
            }?,
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
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;
                        let right_value = get_integer(
                            right,
                            &compiler.program.constants,
                            &ssa_registers,
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

                Ok(())
            }?,
            Operation::ADD => {
                let Add {
                    destination,
                    left,
                    right,
                    r#type,
                } = Add::from(*current_instruction);
                let destination_index = destination.index as usize;
                let sum = match r#type {
                    OperandType::INTEGER => {
                        let left_value = get_integer(
                            left,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;
                        let right_value = get_integer(
                            right,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;

                        function_builder.ins().iadd(left_value, right_value)
                    }
                    OperandType::STRING => {
                        let left_value = get_string(
                            left,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                            allocate_string_function,
                            thread_context,
                        )?;
                        let right_value = get_string(
                            right,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                            allocate_string_function,
                            thread_context,
                        )?;

                        let concatenate_strings_function = {
                            let mut concatenate_strings_signature =
                                Signature::new(compiler.module.isa().default_call_conv());

                            concatenate_strings_signature.params.extend([
                                AbiParam::new(pointer_type),
                                AbiParam::new(pointer_type),
                                AbiParam::new(pointer_type),
                            ]);
                            concatenate_strings_signature
                                .returns
                                .push(AbiParam::new(I64));

                            compiler.declare_imported_function(
                                &mut function_builder,
                                "concatenate_strings",
                                concatenate_strings_signature,
                            )?
                        };

                        let call_instruction = function_builder.ins().call(
                            concatenate_strings_function,
                            &[left_value, right_value, thread_context],
                        );

                        function_builder.inst_results(call_instruction)[0]
                    }
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                ssa_registers[destination_index] = sum;

                function_builder.ins().jump(instruction_blocks[ip + 1], &[]);

                Ok(())
            }?,
            Operation::CALL => {
                let Call {
                    destination_index,
                    prototype_index,
                    arguments_start,
                    argument_count,
                    return_type: _,
                } = Call::from(*current_instruction);
                let callee_function_ids = compiler
                    .function_ids
                    .get(prototype_index as usize)
                    .ok_or(JitError::FunctionIndexOutOfBounds {
                        ip,
                        function_index: prototype_index,
                        total_function_count: compiler.function_ids.len(),
                    })?;
                let FunctionIds::Other { direct, .. } = callee_function_ids else {
                    unreachable!();
                };
                let callee_function_reference = compiler
                    .module
                    .declare_func_in_func(*direct, function_builder.func);
                let arguments_end = arguments_start + argument_count;
                let arguments_range = arguments_start as usize..arguments_end as usize;

                let call_arguments_list = chunk.call_arguments.get(arguments_range).ok_or(
                    JitError::ArgumentsRangeOutOfBounds {
                        arguments_start,
                        arguments_end,
                        total_argument_count: chunk.call_arguments.len(),
                    },
                )?;
                let mut arguments = Vec::with_capacity(call_arguments_list.len() + 1);

                for (address, r#type) in call_arguments_list {
                    let argument_value = match *r#type {
                        OperandType::INTEGER => {
                            let integer_value = get_integer(
                                *address,
                                &compiler.program.constants,
                                &ssa_registers,
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

                arguments.push(thread_context);

                let call_instruction = function_builder
                    .ins()
                    .call(callee_function_reference, &arguments);
                let return_value = function_builder.inst_results(call_instruction)[0];

                ssa_registers[destination_index as usize] = return_value;

                function_builder.ins().jump(instruction_blocks[ip + 1], &[]);
            }
            Operation::CALL_NATIVE => {
                let CallNative {
                    destination,
                    function,
                    arguments_index,
                } = CallNative::from(*current_instruction);

                let function_type = function.r#type();
                let argument_count = function_type.value_parameters.len();
                let arguments_range =
                    arguments_index as usize..(arguments_index as usize + argument_count);
                let call_arguments_list = chunk.call_arguments.get(arguments_range).ok_or(
                    JitError::ArgumentsRangeOutOfBounds {
                        arguments_start: arguments_index,
                        arguments_end: arguments_index + argument_count as u16,
                        total_argument_count: chunk.call_arguments.len(),
                    },
                )?;
                let mut arguments = Vec::with_capacity(call_arguments_list.len() + 1);

                for (address, r#type) in call_arguments_list {
                    let argument_value = match *r#type {
                        OperandType::STRING => get_string(
                            *address,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                            allocate_string_function,
                            thread_context,
                        )?,
                        _ => {
                            return Err(JitError::UnsupportedOperandType {
                                operand_type: *r#type,
                            });
                        }
                    };

                    arguments.push(argument_value);
                }

                arguments.push(thread_context);

                let function_reference = match function.name() {
                    "read_line" => read_line_function,
                    "write_line" => write_line_function,
                    _ => {
                        return Err(JitError::UnhandledNativeFunction {
                            function_name: function.name().to_string(),
                        });
                    }
                };

                let call_instruction = function_builder.ins().call(function_reference, &arguments);

                if function_type.return_type != Type::None {
                    let return_value = function_builder.inst_results(call_instruction)[0];

                    ssa_registers[destination.index as usize] = return_value;
                }

                function_builder.ins().jump(instruction_blocks[ip + 1], &[]);
            }
            Operation::JUMP => {
                let Jump {
                    offset,
                    is_positive,
                } = Jump::from(*current_instruction);
                let offset = offset + 1;
                let next_ip = if is_positive {
                    ip + offset as usize
                } else {
                    ip - offset as usize
                };

                function_builder
                    .ins()
                    .jump(instruction_blocks[next_ip], &[]);
            }
            Operation::RETURN => {
                let Return {
                    should_return_value,
                    return_value_address,
                    r#type,
                } = Return::from(*current_instruction);

                if should_return_value {
                    let value_to_return = match r#type {
                        OperandType::INTEGER => get_integer(
                            return_value_address,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?,
                        _ => {
                            return Err(JitError::UnsupportedOperandType {
                                operand_type: r#type,
                            });
                        }
                    };

                    function_builder.ins().return_(&[value_to_return]);
                } else {
                    let zero = function_builder.ins().iconst(I64, 0);

                    function_builder.ins().return_(&[zero]);
                }
            }
            _ => {
                return Err(JitError::UnhandledOperation { operation });
            }
        }
    }

    function_builder.switch_to_block(return_block);
    function_builder.append_block_param(return_block, I64);
    function_builder.append_block_params_for_function_returns(return_block);

    let return_value = function_builder.block_params(return_block)[0];

    function_builder.ins().return_(&[return_value]);
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
                let cranelift_ir = compilation_context.func.display().to_string();

                JitError::CompilationError {
                    message,
                    cranelift_ir,
                }
            } else {
                let cranelift_ir = compilation_context.func.display().to_string();

                JitError::CompilationError {
                    message: error.to_string(),
                    cranelift_ir,
                }
            }
        })?;

    compiler.module.clear_context(&mut compilation_context);

    Ok(())
}

fn get_integer(
    address: Address,
    constants: &ConstantTable,
    ssa_registers: &[CraneliftValue],
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    match address.memory {
        MemoryKind::REGISTER => Ok(ssa_registers[address.index as usize]),
        MemoryKind::CONSTANT => {
            let integer =
                constants
                    .get_integer(address.index)
                    .ok_or(JitError::ConstantIndexOutOfBounds {
                        constant_index: address.index,
                        total_constant_count: constants.len(),
                    })?;

            Ok(function_builder.ins().iconst(I64, integer))
        }
        _ => Err(JitError::UnsupportedMemoryKind {
            memory_kind: address.memory,
        }),
    }
}

fn get_string(
    address: Address,
    constants: &ConstantTable,
    ssa_registers: &[CraneliftValue],
    function_builder: &mut FunctionBuilder,
    allocate_strings_function: FuncRef,
    thread_context: CraneliftValue,
) -> Result<CraneliftValue, JitError> {
    match address.memory {
        MemoryKind::REGISTER => Ok(ssa_registers[address.index as usize]),
        MemoryKind::CONSTANT => {
            let (string_pointer, string_length) = constants
                .get_string_raw_parts(address.index)
                .ok_or(JitError::ConstantIndexOutOfBounds {
                    constant_index: address.index,
                    total_constant_count: constants.len(),
                })?;
            let string_pointer = function_builder.ins().iconst(I64, string_pointer as i64);
            let string_length = function_builder.ins().iconst(I64, string_length as i64);
            let string_object_pointer = function_builder.ins().call(
                allocate_strings_function,
                &[string_pointer, string_length, thread_context],
            );
            let string_object = function_builder.inst_results(string_object_pointer)[0];

            Ok(string_object)
        }
        _ => Err(JitError::UnsupportedMemoryKind {
            memory_kind: address.memory,
        }),
    }
}
