use cranelift::{
    codegen::CodegenError,
    prelude::{
        AbiParam, FunctionBuilder, FunctionBuilderContext, InstBuilder, IntCC,
        Value as CraneliftValue, types::I64,
    },
};
use cranelift_module::{FuncId, Module, ModuleError};
use tracing::info;

use crate::{
    Chunk, JitCompiler, JitError, MemoryKind, OperandType, Operation,
    instruction::{Add, Call, Jump, Load, Return, Subtract},
};

pub fn compile_direct_function(
    compiler: &mut JitCompiler,
    function_id: FuncId,
    chunk: &Chunk,
) -> Result<(), JitError> {
    info!(
        "Compiling direct function {}",
        chunk.name.as_ref().map_or("anonymous", |path| path.inner())
    );

    let mut function_builder_context = FunctionBuilderContext::new();
    let mut compilation_context = compiler.module.make_context();

    for _ in 0..chunk.r#type.value_parameters.len() {
        compilation_context
            .func
            .signature
            .params
            .push(AbiParam::new(I64));
    }

    compilation_context
        .func
        .signature
        .returns
        .push(AbiParam::new(I64));

    let mut function_builder =
        FunctionBuilder::new(&mut compilation_context.func, &mut function_builder_context);

    #[cfg(debug_assertions)]
    let log_operation_function = {
        use cranelift::prelude::{Signature, types::I8};

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

    for _ in 0..instruction_count {
        let block = function_builder.create_block();

        instruction_blocks.push(block);
    }

    function_builder.switch_to_block(function_entry_block);
    function_builder.append_block_params_for_function_params(function_entry_block);

    let function_arguments = function_builder.block_params(function_entry_block).to_vec();
    let register_count = function_arguments.len() + chunk.register_tags.len();
    let mut ssa_registers = vec![CraneliftValue::from_u32(0); register_count];

    for (index, argument) in function_arguments.iter().enumerate() {
        ssa_registers[index] = *argument;
    }

    function_builder.ins().jump(instruction_blocks[0], &[]);

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
                let destination_index = destination.index as usize;
                let operand_index = operand.index as usize;
                let value = match r#type {
                    OperandType::INTEGER => {
                        let jit_value = match operand.memory {
                            MemoryKind::REGISTER => ssa_registers
                                .get(operand_index)
                                .copied()
                                .ok_or(JitError::RegisterIndexOutOfBounds {
                                    register_index: destination_index,
                                    total_register_count: function_arguments.len(),
                                })?,
                            MemoryKind::CONSTANT => {
                                match chunk.constants[operand_index].as_integer() {
                                    Some(integer) => function_builder.ins().iconst(I64, integer),
                                    None => {
                                        return Err(JitError::InvalidConstantType {
                                            constant_index: operand_index,
                                            expected_type: OperandType::INTEGER,
                                        });
                                    }
                                }
                            }
                            _ => {
                                return Err(JitError::UnsupportedMemoryKind {
                                    memory_kind: operand.memory,
                                });
                            }
                        };

                        Ok(jit_value)
                    }?,
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                ssa_registers[destination_index] = value;

                if jump_next {
                    compiler.emit_jump(ip, 2, &mut function_builder, &[])?;
                }

                Ok(())
            }?,
            Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL => {
                let comparator = current_instruction.a_field();
                let left = current_instruction.b_address();
                let left_index = left.index as usize;
                let right = current_instruction.c_address();
                let right_index = right.index as usize;
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
                        let left_value = match left.memory {
                            MemoryKind::REGISTER => ssa_registers.get(left_index).copied().ok_or(
                                JitError::RegisterIndexOutOfBounds {
                                    register_index: left_index,
                                    total_register_count: function_arguments.len(),
                                },
                            )?,
                            MemoryKind::CONSTANT => {
                                match chunk.constants[left_index].as_integer() {
                                    Some(integer) => function_builder.ins().iconst(I64, integer),
                                    None => {
                                        return Err(JitError::InvalidConstantType {
                                            constant_index: left_index,
                                            expected_type: OperandType::INTEGER,
                                        });
                                    }
                                }
                            }
                            _ => {
                                return Err(JitError::UnsupportedMemoryKind {
                                    memory_kind: left.memory,
                                });
                            }
                        };
                        let right_value = match right.memory {
                            MemoryKind::REGISTER => ssa_registers.get(right_index).copied().ok_or(
                                JitError::RegisterIndexOutOfBounds {
                                    register_index: right_index,
                                    total_register_count: function_arguments.len(),
                                },
                            )?,
                            MemoryKind::CONSTANT => {
                                match chunk.constants[right_index].as_integer() {
                                    Some(integer) => function_builder.ins().iconst(I64, integer),
                                    None => {
                                        return Err(JitError::InvalidConstantType {
                                            constant_index: right_index,
                                            expected_type: OperandType::INTEGER,
                                        });
                                    }
                                }
                            }
                            _ => {
                                return Err(JitError::UnsupportedMemoryKind {
                                    memory_kind: right.memory,
                                });
                            }
                        };

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
                let left_index = left.index as usize;
                let right_index = right.index as usize;
                let sum = match r#type {
                    OperandType::INTEGER => {
                        let left_value = match left.memory {
                            MemoryKind::REGISTER => ssa_registers.get(left_index).copied().ok_or(
                                JitError::RegisterIndexOutOfBounds {
                                    register_index: left_index,
                                    total_register_count: function_arguments.len(),
                                },
                            )?,
                            MemoryKind::CONSTANT => {
                                match chunk.constants[left_index].as_integer() {
                                    Some(integer) => function_builder.ins().iconst(I64, integer),
                                    None => {
                                        return Err(JitError::InvalidConstantType {
                                            constant_index: left_index,
                                            expected_type: OperandType::INTEGER,
                                        });
                                    }
                                }
                            }
                            _ => {
                                return Err(JitError::UnsupportedMemoryKind {
                                    memory_kind: left.memory,
                                });
                            }
                        };
                        let right_value = match right.memory {
                            MemoryKind::REGISTER => ssa_registers.get(right_index).copied().ok_or(
                                JitError::RegisterIndexOutOfBounds {
                                    register_index: right_index,
                                    total_register_count: function_arguments.len(),
                                },
                            )?,
                            MemoryKind::CONSTANT => match chunk.constants[right_index].as_integer()
                            {
                                Some(integer) => function_builder.ins().iconst(I64, integer),
                                None => {
                                    return Err(JitError::InvalidConstantType {
                                        constant_index: right_index,
                                        expected_type: OperandType::INTEGER,
                                    });
                                }
                            },
                            _ => {
                                return Err(JitError::UnsupportedMemoryKind {
                                    memory_kind: right.memory,
                                });
                            }
                        };

                        function_builder.ins().iadd(left_value, right_value)
                    }
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                ssa_registers[destination_index] = sum;

                Ok(())
            }?,
            Operation::SUBTRACT => {
                let Subtract {
                    destination,
                    left,
                    right,
                    r#type,
                } = Subtract::from(*current_instruction);
                let destination_index = destination.index as usize;
                let left_index = left.index as usize;
                let right_index = right.index as usize;
                let difference = match r#type {
                    OperandType::INTEGER => {
                        let left_value = match left.memory {
                            MemoryKind::REGISTER => ssa_registers.get(left_index).copied().ok_or(
                                JitError::RegisterIndexOutOfBounds {
                                    register_index: left_index,
                                    total_register_count: function_arguments.len(),
                                },
                            )?,
                            MemoryKind::CONSTANT => {
                                match chunk.constants[left_index].as_integer() {
                                    Some(integer) => function_builder.ins().iconst(I64, integer),
                                    None => {
                                        return Err(JitError::InvalidConstantType {
                                            constant_index: left_index,
                                            expected_type: OperandType::INTEGER,
                                        });
                                    }
                                }
                            }
                            _ => {
                                return Err(JitError::UnsupportedMemoryKind {
                                    memory_kind: left.memory,
                                });
                            }
                        };
                        let right_value = match right.memory {
                            MemoryKind::REGISTER => ssa_registers.get(right_index).copied().ok_or(
                                JitError::RegisterIndexOutOfBounds {
                                    register_index: right_index,
                                    total_register_count: function_arguments.len(),
                                },
                            )?,
                            MemoryKind::CONSTANT => match chunk.constants[right_index].as_integer()
                            {
                                Some(integer) => function_builder.ins().iconst(I64, integer),
                                None => {
                                    return Err(JitError::InvalidConstantType {
                                        constant_index: right_index,
                                        expected_type: OperandType::INTEGER,
                                    });
                                }
                            },
                            _ => {
                                return Err(JitError::UnsupportedMemoryKind {
                                    memory_kind: right.memory,
                                });
                            }
                        };

                        function_builder.ins().isub(left_value, right_value)
                    }
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                ssa_registers[destination_index] = difference;

                Ok(())
            }?,
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
                    let address_index = address.index as usize;
                    let argument_value = match *r#type {
                        OperandType::INTEGER => {
                            let integer_value = match address.memory {
                                MemoryKind::REGISTER => {
                                    ssa_registers.get(address_index).copied().ok_or(
                                        JitError::RegisterIndexOutOfBounds {
                                            register_index: address_index,
                                            total_register_count: function_arguments.len(),
                                        },
                                    )?
                                }
                                MemoryKind::CONSTANT => {
                                    match chunk.constants[address_index].as_integer() {
                                        Some(integer) => {
                                            function_builder.ins().iconst(I64, integer)
                                        }
                                        None => {
                                            return Err(JitError::InvalidConstantType {
                                                constant_index: address_index,
                                                expected_type: OperandType::INTEGER,
                                            });
                                        }
                                    }
                                }
                                _ => {
                                    return Err(JitError::UnsupportedMemoryKind {
                                        memory_kind: address.memory,
                                    });
                                }
                            };

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

                ssa_registers[destination_index] = return_value;

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
                let return_value_index = return_value_address.index as usize;

                if should_return_value {
                    let value_to_return = match r#type {
                        OperandType::INTEGER => match return_value_address.memory {
                            MemoryKind::REGISTER => ssa_registers
                                .get(return_value_index)
                                .copied()
                                .ok_or(JitError::RegisterIndexOutOfBounds {
                                    register_index: return_value_index,
                                    total_register_count: function_arguments.len(),
                                })?,
                            MemoryKind::CONSTANT => {
                                match chunk.constants[return_value_index].as_integer() {
                                    Some(integer) => function_builder.ins().iconst(I64, integer),
                                    None => {
                                        return Err(JitError::InvalidConstantType {
                                            constant_index: return_value_index,
                                            expected_type: OperandType::INTEGER,
                                        });
                                    }
                                }
                            }
                            _ => {
                                return Err(JitError::UnsupportedMemoryKind {
                                    memory_kind: return_value_address.memory,
                                });
                            }
                        },
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

                JitError::CraneliftModuleError { message }
            } else {
                JitError::CraneliftModuleError {
                    message: error.to_string(),
                }
            }
        })?;

    info!(
        "Finished compiling direct function {}",
        chunk.name.as_ref().map_or("anonymous", |path| path.inner()),
    );

    compiler.module.clear_context(&mut compilation_context);

    Ok(())
}
