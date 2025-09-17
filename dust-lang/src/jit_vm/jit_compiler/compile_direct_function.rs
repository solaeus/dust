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
    Address, Chunk, MemoryKind, OperandType, Operation,
    instruction::{Add, Call, Jump, Load, Return},
    jit_vm::{JitCompiler, JitError, jit_compiler::FunctionIds},
};

fn get_integer(
    address: Address,
    chunk: &Chunk,
    ssa_registers: &[CraneliftValue],
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    match address.memory {
        MemoryKind::REGISTER => Ok(ssa_registers[address.index as usize]),
        MemoryKind::CONSTANT => {
            let integer = chunk.constants.get_integer(address.index).ok_or(
                JitError::ConstantIndexOutOfBounds {
                    constant_index: address.index,
                    total_constant_count: chunk.constants.len(),
                },
            )?;

            Ok(function_builder.ins().iconst(I64, integer))
        }
        _ => Err(JitError::UnsupportedMemoryKind {
            memory_kind: address.memory,
        }),
    }
}

pub fn compile_direct_function(
    compiler: &mut JitCompiler,
    function_id: FuncId,
    chunk: &Chunk,
) -> Result<(), JitError> {
    info!("Compiling direct function {}", &chunk.name);

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
                    OperandType::INTEGER => {
                        self::get_integer(operand, chunk, &ssa_registers, &mut function_builder)?
                    }
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
                        let left_value =
                            get_integer(left, chunk, &ssa_registers, &mut function_builder)?;
                        let right_value =
                            get_integer(right, chunk, &ssa_registers, &mut function_builder)?;

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
                        let left_value =
                            get_integer(left, chunk, &ssa_registers, &mut function_builder)?;
                        let right_value =
                            get_integer(right, chunk, &ssa_registers, &mut function_builder)?;

                        function_builder.ins().iadd(left_value, right_value)
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
                    destination,
                    prototype_index,
                    arguments_index,
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
                let arguments_count = compiler
                    .program
                    .prototypes
                    .get(prototype_index as usize)
                    .ok_or(JitError::FunctionIndexOutOfBounds {
                        ip,
                        function_index: prototype_index,
                        total_function_count: compiler.program.prototypes.len(),
                    })?
                    .r#type
                    .value_parameters
                    .len();
                let arguments_range =
                    arguments_index as usize..(arguments_index as usize + arguments_count);

                let call_arguments_list = chunk.call_arguments.get(arguments_range).ok_or(
                    JitError::ArgumentsRangeOutOfBounds {
                        arguments_list_start: arguments_index,
                        arguments_list_end: arguments_index + arguments_count as u16,
                        total_argument_count: chunk.call_arguments.len(),
                    },
                )?;
                let mut arguments = Vec::with_capacity(call_arguments_list.len() + 3);

                for (address, r#type) in call_arguments_list {
                    let argument_value = match *r#type {
                        OperandType::INTEGER => {
                            let integer_value = get_integer(
                                *address,
                                chunk,
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

                let call_instruction = function_builder
                    .ins()
                    .call(callee_function_reference, &arguments);
                let return_value = function_builder.inst_results(call_instruction)[0];

                ssa_registers[destination.index as usize] = return_value;

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
                            chunk,
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

    info!("Finished compiling direct function {}", &chunk.name);

    compiler.module.clear_context(&mut compilation_context);

    Ok(())
}
