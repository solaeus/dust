use std::mem::offset_of;

use cranelift::codegen::ir::BlockArg;
use cranelift::{
    codegen::CodegenError,
    prelude::{
        AbiParam, FloatCC, FunctionBuilder, FunctionBuilderContext, InstBuilder, IntCC, MemFlags,
        Value as CraneliftValue,
        types::{F64, I8, I64},
    },
};
use cranelift_module::{FuncId, Module, ModuleError};
use tracing::{Level, info, span};

use crate::{
    constant_table::ConstantTable,
    instruction::{
        Address, Call, CallNative, Drop, GetList, Jump, MemoryKind, Move, Negate, NewList,
        OperandType, Operation, Return, SetList, Test, ToString,
    },
    jit_vm::{
        JitCompiler, JitError, ThreadStatus, jit_compiler::FunctionIds, thread::ThreadContext,
    },
    prototype::Prototype,
    r#type::Type,
};

pub fn compile_direct_function(
    compiler: &mut JitCompiler,
    function_id: FuncId,
    prototype: &Prototype,
) -> Result<(), JitError> {
    let span = span!(Level::INFO, "direct");
    let _enter = span.enter();

    let mut function_builder_context = FunctionBuilderContext::new();
    let mut compilation_context = compiler.module.make_context();
    let pointer_type = compiler.module.target_config().pointer_type();
    let value_parameter_count = prototype.function_type.value_parameters.len();

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

    let bytecode_instructions = &prototype.instructions;
    let instruction_count = bytecode_instructions.len();

    let function_entry_block = function_builder.create_block();
    let mut instruction_blocks = Vec::with_capacity(instruction_count);
    let division_by_zero_block = function_builder.create_block();
    let return_block = function_builder.create_block();

    for _ in 0..instruction_count {
        let block = function_builder.create_block();

        for _ in 0..prototype.register_count {
            function_builder.append_block_param(block, I64);
        }

        instruction_blocks.push(block);
    }

    function_builder.switch_to_block(function_entry_block);
    function_builder.append_block_params_for_function_params(function_entry_block);

    let entry_block_params = function_builder.block_params(function_entry_block);
    let thread_context = *entry_block_params.last().unwrap();

    let function_arguments = entry_block_params[..entry_block_params.len() - 1].to_vec();
    let mut ssa_registers = Vec::with_capacity(prototype.register_count as usize);

    for register_index in 0..prototype.register_count as usize {
        if register_index < function_arguments.len() {
            ssa_registers.push(function_arguments[register_index]);
        } else {
            let zero_value = function_builder.ins().iconst(I64, 0);
            ssa_registers.push(zero_value);
        }
    }

    let block_arguments: Vec<BlockArg> = ssa_registers
        .iter()
        .map(|value| BlockArg::Value(*value))
        .collect();

    function_builder
        .ins()
        .jump(instruction_blocks[0], &block_arguments);

    for ip in 0..instruction_count {
        let current_instruction = &bytecode_instructions[ip];
        let operation = current_instruction.operation();
        let instruction_block = instruction_blocks[ip];

        function_builder.switch_to_block(instruction_block);

        let block_parameters = function_builder.block_params(instruction_block);

        for (index, parameter) in block_parameters.iter().enumerate() {
            ssa_registers[index] = *parameter;
        }

        info!("Compiling {operation} at IP {ip} in direct function");

        #[cfg(debug_assertions)]
        {
            let operation_code_instruction = function_builder.ins().iconst(I8, operation.0 as i64);
            let ip_instruction = function_builder.ins().iconst(I64, ip as i64);
            let log_operation_and_ip_function =
                compiler.get_log_operation_and_ip_function(&mut function_builder)?;

            function_builder.ins().call(
                log_operation_and_ip_function,
                &[operation_code_instruction, ip_instruction],
            );
        }

        match operation {
            Operation::MOVE => {
                let Move {
                    destination,
                    operand,
                    r#type,
                    jump_distance,
                    jump_is_positive,
                } = Move::from(*current_instruction);
                let value = match r#type {
                    OperandType::BOOLEAN => get_boolean(
                        operand,
                        &compiler.program.constants,
                        &ssa_registers,
                        &mut function_builder,
                    )?,
                    OperandType::BYTE => get_byte(operand, &ssa_registers, &mut function_builder)?,
                    OperandType::CHARACTER => get_character(
                        operand,
                        &compiler.program.constants,
                        &ssa_registers,
                        &mut function_builder,
                    )?,
                    OperandType::FLOAT => get_float(
                        operand,
                        &compiler.program.constants,
                        &ssa_registers,
                        &mut function_builder,
                    )?,
                    OperandType::INTEGER => self::get_integer(
                        operand,
                        &compiler.program.constants,
                        &ssa_registers,
                        &mut function_builder,
                    )?,
                    OperandType::STRING => get_string(
                        operand,
                        &compiler.program.constants,
                        &ssa_registers,
                        compiler,
                        &mut function_builder,
                        thread_context,
                    )?,
                    OperandType::LIST_BOOLEAN
                    | OperandType::LIST_BYTE
                    | OperandType::LIST_CHARACTER
                    | OperandType::LIST_FLOAT
                    | OperandType::LIST_INTEGER
                    | OperandType::LIST_STRING
                    | OperandType::LIST_LIST
                    | OperandType::LIST_FUNCTION => get_list(operand.index, &ssa_registers),
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                let stored_value = match r#type {
                    OperandType::FLOAT => {
                        function_builder.ins().bitcast(I64, MemFlags::new(), value)
                    }
                    _ => value,
                };

                ssa_registers[destination as usize] = stored_value;

                if jump_distance > 0 {
                    let distance = (jump_distance + 1) as usize;
                    let block_arguments: Vec<BlockArg> = ssa_registers
                        .iter()
                        .map(|value| BlockArg::Value(*value))
                        .collect();

                    if jump_is_positive {
                        function_builder
                            .ins()
                            .jump(instruction_blocks[ip + distance], &block_arguments);
                    } else {
                        function_builder
                            .ins()
                            .jump(instruction_blocks[ip - distance], &block_arguments);
                    }
                } else {
                    let block_arguments: Vec<BlockArg> = ssa_registers
                        .iter()
                        .map(|value| BlockArg::Value(*value))
                        .collect();

                    function_builder
                        .ins()
                        .jump(instruction_blocks[ip + 1], &block_arguments);
                }

                Ok(())
            }?,
            Operation::DROP => {
                let Drop {
                    drop_list_start: _,
                    drop_list_end: _,
                } = Drop::from(*current_instruction);

                // In SSA-based direct compilation, DROP is a no-op since we don't use register tags
                // The garbage collector handles cleanup based on liveness analysis

                let block_arguments: Vec<BlockArg> = ssa_registers
                    .iter()
                    .map(|value| BlockArg::Value(*value))
                    .collect();

                function_builder
                    .ins()
                    .jump(instruction_blocks[ip + 1], &block_arguments);
            }
            Operation::NEW_LIST => {
                let NewList {
                    destination,
                    initial_length,
                    list_type,
                } = NewList::from(*current_instruction);
                let list_type_value = function_builder.ins().iconst(I8, list_type.0 as i64);
                let list_length_value = function_builder.ins().iconst(I64, initial_length as i64);
                let zero = function_builder.ins().iconst(I64, 0);
                let allocate_list_function =
                    compiler.get_allocate_list_function(&mut function_builder)?;

                let call_allocate_list_instruction = function_builder.ins().call(
                    allocate_list_function,
                    &[
                        list_type_value,
                        list_length_value,
                        thread_context,
                        zero,
                        zero,
                    ],
                );
                let list_object_pointer =
                    function_builder.inst_results(call_allocate_list_instruction)[0];

                ssa_registers[destination as usize] = list_object_pointer;

                let block_arguments: Vec<BlockArg> = ssa_registers
                    .iter()
                    .map(|value| BlockArg::Value(*value))
                    .collect();

                function_builder
                    .ins()
                    .jump(instruction_blocks[ip + 1], &block_arguments);
            }
            Operation::SET_LIST => {
                let SetList {
                    destination_list,
                    item_source,
                    list_index,
                    item_type,
                } = SetList::from(*current_instruction);
                let list_pointer = get_list(destination_list, &ssa_registers);
                let item_value = match item_type {
                    OperandType::INTEGER => get_integer(
                        item_source,
                        &compiler.program.constants,
                        &ssa_registers,
                        &mut function_builder,
                    )?,
                    OperandType::BOOLEAN => get_boolean(
                        item_source,
                        &compiler.program.constants,
                        &ssa_registers,
                        &mut function_builder,
                    )?,
                    OperandType::BYTE => {
                        get_byte(item_source, &ssa_registers, &mut function_builder)?
                    }
                    OperandType::CHARACTER => get_character(
                        item_source,
                        &compiler.program.constants,
                        &ssa_registers,
                        &mut function_builder,
                    )?,
                    OperandType::FLOAT => get_float(
                        item_source,
                        &compiler.program.constants,
                        &ssa_registers,
                        &mut function_builder,
                    )?,
                    OperandType::STRING => get_string(
                        item_source,
                        &compiler.program.constants,
                        &ssa_registers,
                        compiler,
                        &mut function_builder,
                        thread_context,
                    )?,
                    OperandType::LIST_BOOLEAN
                    | OperandType::LIST_BYTE
                    | OperandType::LIST_CHARACTER
                    | OperandType::LIST_FLOAT
                    | OperandType::LIST_INTEGER
                    | OperandType::LIST_STRING
                    | OperandType::LIST_LIST
                    | OperandType::LIST_FUNCTION => get_list(item_source.index, &ssa_registers),
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: item_type,
                        });
                    }
                };
                let list_index = function_builder.ins().iconst(I64, list_index as i64);

                let item_value_as_i64 = match item_type {
                    OperandType::FLOAT => {
                        function_builder
                            .ins()
                            .bitcast(I64, MemFlags::new(), item_value)
                    }
                    _ => item_value,
                };
                let instert_into_list_function =
                    compiler.get_insert_into_list_function(&mut function_builder)?;

                function_builder.ins().call(
                    instert_into_list_function,
                    &[list_pointer, list_index, item_value_as_i64],
                );

                let block_arguments: Vec<BlockArg> = ssa_registers
                    .iter()
                    .map(|value| BlockArg::Value(*value))
                    .collect();

                function_builder
                    .ins()
                    .jump(instruction_blocks[ip + 1], &block_arguments);
            }
            Operation::GET_LIST => {
                let GetList {
                    destination,
                    list,
                    list_index,
                    ..
                } = GetList::from(*current_instruction);

                let list_pointer = get_list(list.index, &ssa_registers);
                let list_index = get_integer(
                    list_index,
                    &compiler.program.constants,
                    &ssa_registers,
                    &mut function_builder,
                )?;
                let get_from_list_function =
                    compiler.get_get_from_list_function(&mut function_builder)?;

                let call_get_list_instruction = function_builder.ins().call(
                    get_from_list_function,
                    &[list_pointer, list_index, thread_context],
                );
                let element_value = function_builder.inst_results(call_get_list_instruction)[0];

                ssa_registers[destination as usize] = element_value;

                let block_arguments: Vec<BlockArg> = ssa_registers
                    .iter()
                    .map(|value| BlockArg::Value(*value))
                    .collect();

                function_builder
                    .ins()
                    .jump(instruction_blocks[ip + 1], &block_arguments);
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
                    OperandType::BOOLEAN => {
                        let left_value = get_boolean(
                            left,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;
                        let right_value = get_boolean(
                            right,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;

                        function_builder
                            .ins()
                            .icmp(comparison, left_value, right_value)
                    }
                    OperandType::BYTE => {
                        let left_value = get_byte(left, &ssa_registers, &mut function_builder)?;
                        let right_value = get_byte(right, &ssa_registers, &mut function_builder)?;

                        function_builder
                            .ins()
                            .icmp(comparison, left_value, right_value)
                    }
                    OperandType::CHARACTER => {
                        let left_value = get_character(
                            left,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;
                        let right_value = get_character(
                            right,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;

                        function_builder
                            .ins()
                            .icmp(comparison, left_value, right_value)
                    }
                    OperandType::FLOAT => {
                        let comparison = match comparison {
                            IntCC::Equal => FloatCC::Equal,
                            IntCC::NotEqual => FloatCC::NotEqual,
                            IntCC::SignedLessThan => FloatCC::LessThan,
                            IntCC::SignedGreaterThanOrEqual => FloatCC::GreaterThanOrEqual,
                            IntCC::SignedLessThanOrEqual => FloatCC::LessThanOrEqual,
                            IntCC::SignedGreaterThan => FloatCC::GreaterThan,
                            _ => {
                                unreachable!();
                            }
                        };
                        let left_value = get_float(
                            left,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;
                        let right_value = get_float(
                            right,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;

                        function_builder
                            .ins()
                            .fcmp(comparison, left_value, right_value)
                    }
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
                    OperandType::STRING => {
                        let left_pointer = get_string(
                            left,
                            &compiler.program.constants,
                            &ssa_registers,
                            compiler,
                            &mut function_builder,
                            thread_context,
                        )?;
                        let right_pointer = get_string(
                            right,
                            &compiler.program.constants,
                            &ssa_registers,
                            compiler,
                            &mut function_builder,
                            thread_context,
                        )?;
                        let compare_function = match operation {
                            Operation::EQUAL => compiler
                                .get_compare_strings_equal_function(&mut function_builder)?,
                            Operation::LESS => compiler
                                .get_compare_strings_less_than_function(&mut function_builder)?,
                            Operation::LESS_EQUAL => compiler
                                .get_compare_strings_less_than_equal_function(
                                    &mut function_builder,
                                )?,
                            _ => {
                                return Err(JitError::UnhandledOperation { operation });
                            }
                        };

                        let call_instruction = function_builder
                            .ins()
                            .call(compare_function, &[left_pointer, right_pointer]);
                        let comparison_result = function_builder.inst_results(call_instruction)[0];

                        if comparator != 0 {
                            function_builder
                                .ins()
                                .icmp_imm(IntCC::Equal, comparison_result, 1)
                        } else {
                            function_builder
                                .ins()
                                .icmp_imm(IntCC::Equal, comparison_result, 0)
                        }
                    }
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                let block_arguments: Vec<BlockArg> = ssa_registers
                    .iter()
                    .map(|value| BlockArg::Value(*value))
                    .collect();

                function_builder.ins().brif(
                    comparison_result,
                    instruction_blocks[ip + 2],
                    &block_arguments,
                    instruction_blocks[ip + 1],
                    &block_arguments,
                );

                Ok(())
            }?,
            Operation::ADD
            | Operation::SUBTRACT
            | Operation::MULTIPLY
            | Operation::DIVIDE
            | Operation::MODULO
            | Operation::POWER => {
                let destination = current_instruction.destination();
                let left = current_instruction.b_address();
                let right = current_instruction.c_address();
                let r#type = current_instruction.operand_type();

                let result_value = match r#type {
                    OperandType::BYTE => {
                        let left_value = get_byte(left, &ssa_registers, &mut function_builder)?;
                        let right_value = get_byte(right, &ssa_registers, &mut function_builder)?;

                        match operation {
                            Operation::ADD => function_builder.ins().iadd(left_value, right_value),
                            Operation::SUBTRACT => {
                                function_builder.ins().isub(left_value, right_value)
                            }
                            Operation::MULTIPLY => {
                                function_builder.ins().imul(left_value, right_value)
                            }
                            Operation::DIVIDE => {
                                let division_block = function_builder.create_block();
                                let right_is_zero =
                                    function_builder
                                        .ins()
                                        .icmp_imm(IntCC::Equal, right_value, 0);

                                function_builder.ins().brif(
                                    right_is_zero,
                                    division_by_zero_block,
                                    &[],
                                    division_block,
                                    &[],
                                );
                                function_builder.switch_to_block(division_block);
                                function_builder.ins().udiv(left_value, right_value)
                            }
                            Operation::MODULO => {
                                let modulo_block = function_builder.create_block();
                                let right_is_zero =
                                    function_builder
                                        .ins()
                                        .icmp_imm(IntCC::Equal, right_value, 0);

                                function_builder.ins().brif(
                                    right_is_zero,
                                    division_by_zero_block,
                                    &[],
                                    modulo_block,
                                    &[],
                                );
                                function_builder.switch_to_block(modulo_block);
                                function_builder.ins().urem(left_value, right_value)
                            }
                            _ => {
                                return Err(JitError::UnhandledOperation { operation });
                            }
                        }
                    }
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

                        match operation {
                            Operation::ADD => function_builder.ins().iadd(left_value, right_value),
                            Operation::SUBTRACT => {
                                function_builder.ins().isub(left_value, right_value)
                            }
                            Operation::MULTIPLY => {
                                function_builder.ins().imul(left_value, right_value)
                            }
                            Operation::DIVIDE => {
                                let division_block = function_builder.create_block();
                                let right_is_zero =
                                    function_builder
                                        .ins()
                                        .icmp_imm(IntCC::Equal, right_value, 0);

                                function_builder.ins().brif(
                                    right_is_zero,
                                    division_by_zero_block,
                                    &[],
                                    division_block,
                                    &[],
                                );
                                function_builder.switch_to_block(division_block);
                                function_builder.ins().sdiv(left_value, right_value)
                            }
                            Operation::MODULO => {
                                let modulo_block = function_builder.create_block();
                                let right_is_zero =
                                    function_builder
                                        .ins()
                                        .icmp_imm(IntCC::Equal, right_value, 0);

                                function_builder.ins().brif(
                                    right_is_zero,
                                    division_by_zero_block,
                                    &[],
                                    modulo_block,
                                    &[],
                                );
                                function_builder.switch_to_block(modulo_block);
                                function_builder.ins().srem(left_value, right_value)
                            }
                            Operation::POWER => {
                                let integer_power_function =
                                    compiler.get_integer_power_function(&mut function_builder)?;
                                let call_instruction = function_builder
                                    .ins()
                                    .call(integer_power_function, &[left_value, right_value]);

                                function_builder.inst_results(call_instruction)[0]
                            }
                            _ => {
                                return Err(JitError::UnhandledOperation { operation });
                            }
                        }
                    }
                    OperandType::FLOAT => {
                        let left_value = get_float(
                            left,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;
                        let right_value = get_float(
                            right,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;

                        match operation {
                            Operation::ADD => function_builder.ins().fadd(left_value, right_value),
                            Operation::SUBTRACT => {
                                function_builder.ins().fsub(left_value, right_value)
                            }
                            Operation::MULTIPLY => {
                                function_builder.ins().fmul(left_value, right_value)
                            }
                            Operation::DIVIDE => {
                                let division_block = function_builder.create_block();
                                let zero = function_builder.ins().f64const(0.0);
                                let right_is_zero =
                                    function_builder
                                        .ins()
                                        .fcmp(FloatCC::Equal, right_value, zero);

                                function_builder.ins().brif(
                                    right_is_zero,
                                    division_by_zero_block,
                                    &[],
                                    division_block,
                                    &[],
                                );
                                function_builder.switch_to_block(division_block);
                                function_builder.ins().fdiv(left_value, right_value)
                            }
                            Operation::MODULO => {
                                let modulo_block = function_builder.create_block();
                                let zero = function_builder.ins().f64const(0.0);
                                let right_is_zero =
                                    function_builder
                                        .ins()
                                        .fcmp(FloatCC::Equal, right_value, zero);

                                function_builder.ins().brif(
                                    right_is_zero,
                                    division_by_zero_block,
                                    &[],
                                    modulo_block,
                                    &[],
                                );
                                function_builder.switch_to_block(modulo_block);

                                let divided = function_builder.ins().fdiv(left_value, right_value);
                                let truncated = function_builder.ins().floor(divided);
                                let multiplied =
                                    function_builder.ins().fmul(truncated, right_value);

                                function_builder.ins().fsub(left_value, multiplied)
                            }
                            Operation::POWER => {
                                let float_power_function =
                                    compiler.get_float_power_function(&mut function_builder)?;
                                let call_instruction = function_builder
                                    .ins()
                                    .call(float_power_function, &[left_value, right_value]);

                                function_builder.inst_results(call_instruction)[0]
                            }
                            _ => {
                                return Err(JitError::UnhandledOperation { operation });
                            }
                        }
                    }
                    OperandType::STRING => {
                        if operation != Operation::ADD {
                            return Err(JitError::UnhandledOperation { operation });
                        }

                        let left_pointer = get_string(
                            left,
                            &compiler.program.constants,
                            &ssa_registers,
                            compiler,
                            &mut function_builder,
                            thread_context,
                        )?;
                        let right_pointer = get_string(
                            right,
                            &compiler.program.constants,
                            &ssa_registers,
                            compiler,
                            &mut function_builder,
                            thread_context,
                        )?;
                        let concatenate_strings_function =
                            compiler.get_concatenate_strings_function(&mut function_builder)?;
                        let call_instruction = function_builder.ins().call(
                            concatenate_strings_function,
                            &[left_pointer, right_pointer, thread_context],
                        );

                        function_builder.inst_results(call_instruction)[0]
                    }
                    OperandType::CHARACTER_STRING => {
                        if operation != Operation::ADD {
                            return Err(JitError::UnhandledOperation { operation });
                        }

                        let left_value = get_character(
                            left,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;
                        let right_pointer = get_string(
                            right,
                            &compiler.program.constants,
                            &ssa_registers,
                            compiler,
                            &mut function_builder,
                            thread_context,
                        )?;
                        let concatenate_character_string_function = compiler
                            .get_concatenate_character_string_function(&mut function_builder)?;
                        let call_instruction = function_builder.ins().call(
                            concatenate_character_string_function,
                            &[left_value, right_pointer, thread_context],
                        );

                        function_builder.inst_results(call_instruction)[0]
                    }
                    OperandType::STRING_CHARACTER => {
                        if operation != Operation::ADD {
                            return Err(JitError::UnhandledOperation { operation });
                        }

                        let left_pointer = get_string(
                            left,
                            &compiler.program.constants,
                            &ssa_registers,
                            compiler,
                            &mut function_builder,
                            thread_context,
                        )?;
                        let right_value = get_character(
                            right,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;
                        let concatenate_string_character_function = compiler
                            .get_concatenate_string_character_function(&mut function_builder)?;
                        let call_instruction = function_builder.ins().call(
                            concatenate_string_character_function,
                            &[left_pointer, right_value, thread_context],
                        );

                        function_builder.inst_results(call_instruction)[0]
                    }
                    OperandType::CHARACTER => {
                        if operation != Operation::ADD {
                            return Err(JitError::UnhandledOperation { operation });
                        }

                        let left_value = get_character(
                            left,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;
                        let right_value = get_character(
                            right,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;
                        let concatenate_characters_function =
                            compiler.get_concatenate_characters_function(&mut function_builder)?;

                        let call_instruction = function_builder.ins().call(
                            concatenate_characters_function,
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

                let stored_value = match r#type {
                    OperandType::FLOAT => {
                        function_builder
                            .ins()
                            .bitcast(I64, MemFlags::new(), result_value)
                    }
                    _ => result_value,
                };

                ssa_registers[destination.index as usize] = stored_value;

                let block_arguments: Vec<BlockArg> = ssa_registers
                    .iter()
                    .map(|value| BlockArg::Value(*value))
                    .collect();

                function_builder
                    .ins()
                    .jump(instruction_blocks[ip + 1], &block_arguments);
            }
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

                let call_arguments_list = prototype.call_arguments.get(arguments_range).ok_or(
                    JitError::ArgumentsRangeOutOfBounds {
                        arguments_start,
                        arguments_end,
                        total_argument_count: prototype.call_arguments.len(),
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

                let block_arguments: Vec<BlockArg> = ssa_registers
                    .iter()
                    .map(|value| BlockArg::Value(*value))
                    .collect();

                function_builder
                    .ins()
                    .jump(instruction_blocks[ip + 1], &block_arguments);
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
                let call_arguments_list = prototype.call_arguments.get(arguments_range).ok_or(
                    JitError::ArgumentsRangeOutOfBounds {
                        arguments_start: arguments_index,
                        arguments_end: arguments_index + argument_count as u16,
                        total_argument_count: prototype.call_arguments.len(),
                    },
                )?;
                let mut arguments = Vec::with_capacity(call_arguments_list.len() + 1);

                for (address, r#type) in call_arguments_list {
                    let argument_value = match *r#type {
                        OperandType::STRING => get_string(
                            *address,
                            &compiler.program.constants,
                            &ssa_registers,
                            compiler,
                            &mut function_builder,
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
                    "read_line" => compiler.get_read_line_function(&mut function_builder)?,
                    "write_line" => {
                        compiler.get_write_line_string_function(&mut function_builder)?
                    }
                    _ => {
                        return Err(JitError::UnhandledNativeFunction {
                            function_name: function.name().to_string(),
                        });
                    }
                };

                let call_instruction = function_builder.ins().call(function_reference, &arguments);

                if function_type.return_type != Type::None {
                    let return_value = function_builder.inst_results(call_instruction)[0];

                    ssa_registers[destination as usize] = return_value;
                }

                let block_arguments: Vec<BlockArg> = ssa_registers
                    .iter()
                    .map(|value| BlockArg::Value(*value))
                    .collect();

                function_builder
                    .ins()
                    .jump(instruction_blocks[ip + 1], &block_arguments);
            }
            Operation::TEST => {
                let Test {
                    comparator,
                    operand,
                    jump_distance,
                } = Test::from(*current_instruction);

                let operand_value = get_boolean(
                    operand,
                    &compiler.program.constants,
                    &ssa_registers,
                    &mut function_builder,
                )?;
                let comparator_value = function_builder.ins().iconst(I64, comparator as i64);
                let comparison_result =
                    function_builder
                        .ins()
                        .icmp(IntCC::Equal, operand_value, comparator_value);

                let distance = (jump_distance + 1) as usize;
                let block_arguments: Vec<BlockArg> = ssa_registers
                    .iter()
                    .map(|value| BlockArg::Value(*value))
                    .collect();

                function_builder.ins().brif(
                    comparison_result,
                    instruction_blocks[ip + distance],
                    &block_arguments,
                    instruction_blocks[ip + 1],
                    &block_arguments,
                );
            }
            Operation::NEGATE => {
                let Negate {
                    destination,
                    operand,
                    r#type,
                } = Negate::from(*current_instruction);

                let result_value = match r#type {
                    OperandType::BOOLEAN => {
                        let boolean_value = get_boolean(
                            operand,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;
                        let is_zero =
                            function_builder
                                .ins()
                                .icmp_imm(IntCC::Equal, boolean_value, 0);
                        let one = function_builder.ins().iconst(I64, 1);
                        let zero = function_builder.ins().iconst(I64, 0);

                        function_builder.ins().select(is_zero, one, zero)
                    }
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                ssa_registers[destination as usize] = result_value;

                let block_arguments: Vec<BlockArg> = ssa_registers
                    .iter()
                    .map(|value| BlockArg::Value(*value))
                    .collect();

                function_builder
                    .ins()
                    .jump(instruction_blocks[ip + 1], &block_arguments);
            }
            Operation::JUMP => {
                let Jump {
                    offset,
                    is_positive,
                    ..
                } = Jump::from(*current_instruction);
                let offset = offset + 1;
                let next_ip = if is_positive {
                    ip + offset as usize
                } else {
                    ip - offset as usize
                };

                let block_arguments: Vec<BlockArg> = ssa_registers
                    .iter()
                    .map(|value| BlockArg::Value(*value))
                    .collect();

                function_builder
                    .ins()
                    .jump(instruction_blocks[next_ip], &block_arguments);
            }
            Operation::RETURN => {
                let Return {
                    should_return_value,
                    return_value_address,
                    r#type,
                } = Return::from(*current_instruction);

                if should_return_value {
                    let value_to_return = match r#type {
                        OperandType::BOOLEAN => get_boolean(
                            return_value_address,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?,
                        OperandType::BYTE => {
                            get_byte(return_value_address, &ssa_registers, &mut function_builder)?
                        }
                        OperandType::CHARACTER => get_character(
                            return_value_address,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?,
                        OperandType::FLOAT => get_float(
                            return_value_address,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?,
                        OperandType::INTEGER => get_integer(
                            return_value_address,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?,
                        OperandType::STRING => get_string(
                            return_value_address,
                            &compiler.program.constants,
                            &ssa_registers,
                            compiler,
                            &mut function_builder,
                            thread_context,
                        )?,
                        OperandType::LIST_BOOLEAN
                        | OperandType::LIST_BYTE
                        | OperandType::LIST_CHARACTER
                        | OperandType::LIST_FLOAT
                        | OperandType::LIST_INTEGER
                        | OperandType::LIST_STRING
                        | OperandType::LIST_LIST
                        | OperandType::LIST_FUNCTION => {
                            get_list(return_value_address.index, &ssa_registers)
                        }
                        _ => {
                            return Err(JitError::UnsupportedOperandType {
                                operand_type: r#type,
                            });
                        }
                    };

                    let return_value = match r#type {
                        OperandType::FLOAT => {
                            function_builder
                                .ins()
                                .bitcast(I64, MemFlags::new(), value_to_return)
                        }
                        _ => value_to_return,
                    };

                    function_builder.ins().return_(&[return_value]);
                } else {
                    let zero = function_builder.ins().iconst(I64, 0);

                    function_builder.ins().return_(&[zero]);
                }
            }
            Operation::TO_STRING => {
                let ToString {
                    destination,
                    operand,
                    r#type,
                } = ToString::from(*current_instruction);

                let string_value = match r#type {
                    OperandType::INTEGER => {
                        let integer_operand = get_integer(
                            operand,
                            &compiler.program.constants,
                            &ssa_registers,
                            &mut function_builder,
                        )?;
                        let integer_to_string_function =
                            compiler.get_integer_to_string_function(&mut function_builder)?;
                        let call_instruction = function_builder.ins().call(
                            integer_to_string_function,
                            &[integer_operand, thread_context],
                        );

                        function_builder.inst_results(call_instruction)[0]
                    }
                    OperandType::STRING => get_string(
                        operand,
                        &compiler.program.constants,
                        &ssa_registers,
                        compiler,
                        &mut function_builder,
                        thread_context,
                    )?,
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                ssa_registers[destination as usize] = string_value;

                let block_arguments: Vec<BlockArg> = ssa_registers
                    .iter()
                    .map(|value| BlockArg::Value(*value))
                    .collect();

                function_builder
                    .ins()
                    .jump(instruction_blocks[ip + 1], &block_arguments);
            }
            _ => {
                return Err(JitError::UnhandledOperation { operation });
            }
        }
    }

    {
        function_builder.switch_to_block(division_by_zero_block);

        let division_by_zero_status = function_builder
            .ins()
            .iconst(I8, ThreadStatus::ErrorDivisionByZero as i64);

        function_builder.ins().store(
            MemFlags::new(),
            division_by_zero_status,
            thread_context,
            offset_of!(ThreadContext, status) as i32,
        );

        let zero_return_value = function_builder.ins().iconst(I64, 0);

        function_builder.ins().return_(&[zero_return_value]);
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

fn get_byte(
    address: Address,
    ssa_registers: &[CraneliftValue],
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    let jit_value = match address.memory {
        MemoryKind::REGISTER => ssa_registers[address.index as usize],
        MemoryKind::ENCODED => function_builder.ins().iconst(I64, address.index as i64),
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
    constants: &ConstantTable,
    ssa_registers: &[CraneliftValue],
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    match address.memory {
        MemoryKind::REGISTER => Ok(ssa_registers[address.index as usize]),
        MemoryKind::CONSTANT => {
            let character = constants.get_character(address.index).ok_or(
                JitError::ConstantIndexOutOfBounds {
                    constant_index: address.index,
                    total_constant_count: constants.len(),
                },
            )?;

            Ok(function_builder.ins().iconst(I64, character as i64))
        }
        _ => Err(JitError::UnsupportedMemoryKind {
            memory_kind: address.memory,
        }),
    }
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

fn get_float(
    address: Address,
    constants: &ConstantTable,
    ssa_registers: &[CraneliftValue],
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    match address.memory {
        MemoryKind::REGISTER => {
            let i64_value = ssa_registers[address.index as usize];
            Ok(function_builder
                .ins()
                .bitcast(F64, MemFlags::new(), i64_value))
        }
        MemoryKind::CONSTANT => {
            let float =
                constants
                    .get_float(address.index)
                    .ok_or(JitError::ConstantIndexOutOfBounds {
                        constant_index: address.index,
                        total_constant_count: constants.len(),
                    })?;

            Ok(function_builder.ins().f64const(float))
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
    compiler: &mut JitCompiler,
    function_builder: &mut FunctionBuilder,
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
            let allocate_strings_function =
                compiler.get_allocate_string_function(function_builder)?;
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

fn get_boolean(
    address: Address,
    _constants: &ConstantTable,
    ssa_registers: &[CraneliftValue],
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    let jit_value = match address.memory {
        MemoryKind::REGISTER => ssa_registers[address.index as usize],
        MemoryKind::ENCODED => {
            let boolean_value = address.index != 0;

            function_builder.ins().iconst(I64, boolean_value as i64)
        }
        _ => {
            return Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            });
        }
    };

    Ok(jit_value)
}

fn get_list(register: u16, ssa_registers: &[CraneliftValue]) -> CraneliftValue {
    ssa_registers[register as usize]
}
