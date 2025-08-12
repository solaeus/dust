mod functions;
mod jit_error;

use std::mem::transmute;

use functions::*;
pub use jit_error::{JIT_ERROR_TEXT, JitError};

use cranelift::{
    codegen::{
        CodegenError,
        ir::{FuncRef, InstBuilder},
    },
    frontend::Switch,
    prelude::{
        AbiParam, Block, FunctionBuilder, FunctionBuilderContext, IntCC, MemFlags, Signature,
        Value as CraneliftValue,
        types::{I8, I64},
    },
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module, ModuleError};
use tracing::{Level, info};

use crate::{
    Address, Chunk, OperandType, Operation, Program, Register, ThreadStatus,
    instruction::{Add, Call, Jump, Load, MemoryKind, Return, Subtract},
    jit_vm::call_stack::{get_call_frame, get_frame_function_index, push_call_frame},
};

pub struct JitCompiler<'a> {
    module: JITModule,
    program: &'a Program,
    main_function_id: FuncId,
    function_ids: Vec<FunctionIds>,
}

impl<'a> JitCompiler<'a> {
    pub fn new(program: &'a Program) -> Self {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();

        builder.symbol("concatenate_strings", concatenate_strings as *const u8);
        builder.symbol("log_operation", log_operation as *const u8);
        builder.symbol("log_value", log_value as *const u8);
        builder.symbol("log_call_frame", log_call_frame as *const u8);

        let module = JITModule::new(builder);

        Self {
            module,
            program,
            main_function_id: FuncId::from_u32(0),
            function_ids: Vec::with_capacity(program.prototypes.len()),
        }
    }

    pub fn compile(&mut self) -> Result<JitLogic, JitError> {
        let span = tracing::span!(Level::INFO, "JIT_Compiler");
        let _enter = span.enter();

        let loop_pointer = self.compile_loop()?;

        Ok(unsafe { transmute::<*const u8, JitLogic>(loop_pointer) })
    }

    fn compile_loop(&mut self) -> Result<*const u8, JitError> {
        let mut context = self.module.make_context();
        let pointer_type = self.module.isa().pointer_type();
        let mut main_signature = Signature::new(self.module.isa().default_call_conv());

        main_signature.params.extend([
            AbiParam::new(pointer_type),
            AbiParam::new(pointer_type),
            AbiParam::new(pointer_type),
            AbiParam::new(pointer_type),
            AbiParam::new(pointer_type),
        ]);

        self.main_function_id = self
            .module
            .declare_function("main", Linkage::Local, &main_signature)
            .map_err(|error| JitError::CraneliftModuleError {
                message: error.to_string(),
            })?;

        let mut stackless_signature = Signature::new(self.module.isa().default_call_conv());

        stackless_signature
            .params
            .extend([AbiParam::new(pointer_type); 5]);

        for (index, chunk) in self.program.prototypes.iter().enumerate() {
            let name = chunk
                .name
                .as_ref()
                .map_or_else(|| format!("proto_{index}"), |path| path.to_string());
            let direct_name = format!("{name}_direct");
            let stackless_name = format!("{name}_stackless");
            let mut direct_signature = Signature::new(self.module.isa().default_call_conv());

            direct_signature.returns.push(AbiParam::new(I64));

            for _ in 0..chunk.r#type.value_parameters.len() {
                direct_signature.params.push(AbiParam::new(I64));
            }

            let direct_function_id = self
                .module
                .declare_function(&direct_name, Linkage::Local, &direct_signature)
                .map_err(|error| JitError::CraneliftModuleError {
                    message: error.to_string(),
                })?;
            let stackless_function_id = self
                .module
                .declare_function(&stackless_name, Linkage::Local, &stackless_signature)
                .map_err(|error| JitError::CraneliftModuleError {
                    message: error.to_string(),
                })?;

            self.function_ids.push(FunctionIds {
                direct: direct_function_id,
                stackless: stackless_function_id,
            });
        }

        let main_function_reference = {
            let reference = self
                .module
                .declare_func_in_func(self.main_function_id, &mut context.func);

            self.compile_stackless_function(self.main_function_id, &self.program.main_chunk, true)?;

            reference
        };
        let function_references = {
            let mut references = Vec::with_capacity(self.program.prototypes.len());

            for (index, FunctionIds { direct, stackless }) in
                self.function_ids.clone().into_iter().enumerate()
            {
                let direct_reference = self.module.declare_func_in_func(direct, &mut context.func);
                let stackless_reference = self
                    .module
                    .declare_func_in_func(stackless, &mut context.func);

                references.push((direct_reference, stackless_reference));

                let chunk = &self.program.prototypes[index];

                self.compile_direct_function(direct, chunk)?;
                self.compile_stackless_function(stackless, chunk, false)?;
            }

            references
        };

        context.func.signature = stackless_signature;
        context
            .func
            .signature
            .returns
            .push(AbiParam::new(ThreadStatus::CRANELIFT_TYPE));

        let loop_function_id = self
            .module
            .declare_function("loop", Linkage::Local, &context.func.signature)
            .map_err(|error| JitError::CraneliftModuleError {
                message: error.to_string(),
            })?;
        let mut function_builder_context = FunctionBuilderContext::new();
        let mut function_builder =
            FunctionBuilder::new(&mut context.func, &mut function_builder_context);
        let mut switch = Switch::new();

        let entry_block = {
            let block = function_builder.create_block();

            function_builder.append_block_params_for_function_params(block);

            block
        };
        let check_for_empty_call_stack_block = function_builder.create_block();
        let check_for_error_function_index_out_of_bounds_block = function_builder.create_block();
        let loop_block = function_builder.create_block();
        let main_function_block = function_builder.create_block();
        let function_blocks = {
            let mut blocks = Vec::with_capacity(self.program.prototypes.len());

            for index in 0..function_references.len() {
                let block = function_builder.create_block();

                blocks.push(block);
                switch.set_entry(index as u128, block);
            }

            blocks
        };
        let return_block = function_builder.create_block();

        let (
            call_stack_pointer,
            call_stack_length_pointer,
            register_stack_pointer,
            return_register_pointer,
            return_type_pointer,
        ) = {
            function_builder.switch_to_block(entry_block);

            let call_stack_pointer = {
                let argument = function_builder.block_params(entry_block)[0];
                let variable = function_builder.declare_var(pointer_type);

                function_builder.def_var(variable, argument);
                function_builder.use_var(variable)
            };
            let call_stack_length_pointer = {
                let argument = function_builder.block_params(entry_block)[1];
                let variable = function_builder.declare_var(pointer_type);

                function_builder.def_var(variable, argument);
                function_builder.use_var(variable)
            };
            let register_stack_pointer = {
                let argument = function_builder.block_params(entry_block)[2];
                let variable = function_builder.declare_var(pointer_type);

                function_builder.def_var(variable, argument);
                function_builder.use_var(variable)
            };
            let return_register_pointer = {
                let argument = function_builder.block_params(entry_block)[3];
                let variable = function_builder.declare_var(pointer_type);

                function_builder.def_var(variable, argument);
                function_builder.use_var(variable)
            };
            let return_type_pointer = {
                let argument = function_builder.block_params(entry_block)[4];
                let variable = function_builder.declare_var(pointer_type);

                function_builder.def_var(variable, argument);
                function_builder.use_var(variable)
            };

            let zero = function_builder.ins().iconst(I64, 0);
            let register_count = function_builder
                .ins()
                .iconst(I64, self.program.main_chunk.register_tags.len() as i64);
            let null_function_index = function_builder.ins().iconst(I64, u32::MAX as i64);

            push_call_frame(
                zero,
                zero,
                null_function_index,
                zero,
                register_count,
                zero,
                zero,
                call_stack_pointer,
                call_stack_length_pointer,
                &mut function_builder,
            );
            function_builder
                .ins()
                .jump(check_for_empty_call_stack_block, &[]);

            (
                call_stack_pointer,
                call_stack_length_pointer,
                register_stack_pointer,
                return_register_pointer,
                return_type_pointer,
            )
        };

        {
            function_builder.switch_to_block(check_for_empty_call_stack_block);

            let call_stack_length =
                function_builder
                    .ins()
                    .load(I64, MemFlags::new(), call_stack_length_pointer, 0);
            let call_stack_is_empty =
                function_builder
                    .ins()
                    .icmp_imm(IntCC::Equal, call_stack_length, 0);
            let return_thread_status = function_builder
                .ins()
                .iconst(ThreadStatus::CRANELIFT_TYPE, ThreadStatus::Return as i64);

            function_builder.ins().brif(
                call_stack_is_empty,
                return_block,
                &[return_thread_status.into()],
                check_for_error_function_index_out_of_bounds_block,
                &[],
            );
        }

        {
            function_builder.switch_to_block(check_for_error_function_index_out_of_bounds_block);

            let call_stack_length =
                function_builder
                    .ins()
                    .load(I64, MemFlags::new(), call_stack_length_pointer, 0);
            let call_stack_is_empty =
                function_builder
                    .ins()
                    .icmp_imm(IntCC::Equal, call_stack_length, 0);
            let return_thread_status = function_builder.ins().iconst(
                ThreadStatus::CRANELIFT_TYPE,
                ThreadStatus::ErrorFunctionIndexOutOfBounds as i64,
            );

            function_builder.ins().brif(
                call_stack_is_empty,
                return_block,
                &[return_thread_status.into()],
                loop_block,
                &[],
            );
        }

        {
            function_builder.switch_to_block(loop_block);

            let top_call_frame_index = {
                let call_stack_length =
                    function_builder
                        .ins()
                        .load(I64, MemFlags::new(), call_stack_length_pointer, 0);
                let one = function_builder.ins().iconst(I64, 1);

                function_builder.ins().isub(call_stack_length, one)
            };
            let function_index = get_frame_function_index(
                top_call_frame_index,
                call_stack_pointer,
                &mut function_builder,
            );

            switch.emit(&mut function_builder, function_index, main_function_block);
        }

        {
            function_builder.switch_to_block(main_function_block);
            function_builder.ins().call(
                main_function_reference,
                &[
                    call_stack_pointer,
                    call_stack_length_pointer,
                    register_stack_pointer,
                    return_register_pointer,
                    return_type_pointer,
                ],
            );
            function_builder
                .ins()
                .jump(check_for_empty_call_stack_block, &[]);
        }

        {
            for (block, (_direct, stackless)) in function_blocks
                .into_iter()
                .zip(function_references.into_iter())
            {
                function_builder.switch_to_block(block);
                function_builder.ins().call(
                    stackless,
                    &[
                        call_stack_pointer,
                        call_stack_length_pointer,
                        register_stack_pointer,
                        return_register_pointer,
                        return_type_pointer,
                    ],
                );
                function_builder
                    .ins()
                    .jump(check_for_empty_call_stack_block, &[]);
            }
        }

        {
            function_builder.switch_to_block(return_block);
            function_builder.append_block_param(return_block, ThreadStatus::CRANELIFT_TYPE);

            let return_thread_status = function_builder.block_params(return_block)[0];

            function_builder.ins().nop();
            function_builder.ins().return_(&[return_thread_status]);
        }

        function_builder.seal_all_blocks();
        function_builder.finalize();
        self.module
            .define_function(loop_function_id, &mut context)
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
        self.module
            .finalize_definitions()
            .map_err(|error| JitError::CraneliftModuleError {
                message: error.to_string(),
            })?;

        Ok(self.module.get_finalized_function(loop_function_id))
    }

    fn compile_direct_function(
        &mut self,
        function_id: FuncId,
        chunk: &Chunk,
    ) -> Result<(), JitError> {
        info!(
            "Compiling direct function {}",
            chunk.name.as_ref().map_or("anonymous", |path| path.inner())
        );

        let mut function_builder_context = FunctionBuilderContext::new();
        let mut compilation_context = self.module.make_context();

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
            let mut log_operation_signature = Signature::new(self.module.isa().default_call_conv());

            log_operation_signature.params.push(AbiParam::new(I8));
            log_operation_signature.returns = vec![];

            self.declare_imported_function(
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
                let operation_code_instruction =
                    function_builder.ins().iconst(I8, operation.0 as i64);

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
                                MemoryKind::REGISTER => {
                                    ssa_registers.get(operand_index).copied().ok_or(
                                        JitError::RegisterIndexOutOfBounds {
                                            register_index: destination_index,
                                            total_register_count: function_arguments.len(),
                                        },
                                    )?
                                }
                                MemoryKind::CONSTANT => match chunk.constants[operand_index]
                                    .as_integer()
                                {
                                    Some(integer) => function_builder.ins().iconst(I64, integer),
                                    None => {
                                        return Err(JitError::InvalidConstantType {
                                            constant_index: operand_index,
                                            expected_type: OperandType::INTEGER,
                                        });
                                    }
                                },
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
                        self.emit_jump(ip, 2, &mut function_builder, &[])?;
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
                                MemoryKind::REGISTER => ssa_registers
                                    .get(left_index)
                                    .copied()
                                    .ok_or(JitError::RegisterIndexOutOfBounds {
                                        register_index: left_index,
                                        total_register_count: function_arguments.len(),
                                    })?,
                                MemoryKind::CONSTANT => {
                                    match chunk.constants[left_index].as_integer() {
                                        Some(integer) => {
                                            function_builder.ins().iconst(I64, integer)
                                        }
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
                                MemoryKind::REGISTER => ssa_registers
                                    .get(right_index)
                                    .copied()
                                    .ok_or(JitError::RegisterIndexOutOfBounds {
                                        register_index: right_index,
                                        total_register_count: function_arguments.len(),
                                    })?,
                                MemoryKind::CONSTANT => {
                                    match chunk.constants[right_index].as_integer() {
                                        Some(integer) => {
                                            function_builder.ins().iconst(I64, integer)
                                        }
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
                                MemoryKind::REGISTER => ssa_registers
                                    .get(left_index)
                                    .copied()
                                    .ok_or(JitError::RegisterIndexOutOfBounds {
                                        register_index: left_index,
                                        total_register_count: function_arguments.len(),
                                    })?,
                                MemoryKind::CONSTANT => {
                                    match chunk.constants[left_index].as_integer() {
                                        Some(integer) => {
                                            function_builder.ins().iconst(I64, integer)
                                        }
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
                                MemoryKind::REGISTER => ssa_registers
                                    .get(right_index)
                                    .copied()
                                    .ok_or(JitError::RegisterIndexOutOfBounds {
                                        register_index: right_index,
                                        total_register_count: function_arguments.len(),
                                    })?,
                                MemoryKind::CONSTANT => match chunk.constants[right_index]
                                    .as_integer()
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
                                MemoryKind::REGISTER => ssa_registers
                                    .get(left_index)
                                    .copied()
                                    .ok_or(JitError::RegisterIndexOutOfBounds {
                                        register_index: left_index,
                                        total_register_count: function_arguments.len(),
                                    })?,
                                MemoryKind::CONSTANT => match chunk.constants[left_index]
                                    .as_integer()
                                {
                                    Some(integer) => function_builder.ins().iconst(I64, integer),
                                    None => {
                                        return Err(JitError::InvalidConstantType {
                                            constant_index: left_index,
                                            expected_type: OperandType::INTEGER,
                                        });
                                    }
                                },
                                _ => {
                                    return Err(JitError::UnsupportedMemoryKind {
                                        memory_kind: left.memory,
                                    });
                                }
                            };
                            let right_value = match right.memory {
                                MemoryKind::REGISTER => ssa_registers
                                    .get(right_index)
                                    .copied()
                                    .ok_or(JitError::RegisterIndexOutOfBounds {
                                        register_index: right_index,
                                        total_register_count: function_arguments.len(),
                                    })?,
                                MemoryKind::CONSTANT => match chunk.constants[right_index]
                                    .as_integer()
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
                    let callee_function_ids = self.function_ids.get(prototype_index).ok_or(
                        JitError::FunctionIndexOutOfBounds {
                            ip,
                            function_index: prototype_index,
                            total_function_count: self.function_ids.len(),
                        },
                    )?;
                    let callee_function_reference = self
                        .module
                        .declare_func_in_func(callee_function_ids.direct, function_builder.func);

                    let call_arguments_list = chunk
                        .call_argument_lists
                        .get(arguments_index)
                        .ok_or(JitError::ArgumentsIndexOutOfBounds {
                            arguments_index,
                            total_argument_count: chunk.call_argument_lists.len(),
                        })?;
                    let mut arguments = Vec::with_capacity(call_arguments_list.len() + 3);

                    for (address, r#type) in call_arguments_list {
                        let address_index = address.index as usize;
                        let argument_value = match *r#type {
                            OperandType::INTEGER => {
                                let integer_value = match address.memory {
                                    MemoryKind::REGISTER => ssa_registers
                                        .get(address_index)
                                        .copied()
                                        .ok_or(JitError::RegisterIndexOutOfBounds {
                                            register_index: address_index,
                                            total_register_count: function_arguments.len(),
                                        })?,
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
                        self.emit_jump(
                            ip,
                            (offset + 1) as isize,
                            &mut function_builder,
                            &instruction_blocks,
                        )?;
                    } else {
                        self.emit_jump(
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
                                MemoryKind::CONSTANT => match chunk.constants[return_value_index]
                                    .as_integer()
                                {
                                    Some(integer) => function_builder.ins().iconst(I64, integer),
                                    None => {
                                        return Err(JitError::InvalidConstantType {
                                            constant_index: return_value_index,
                                            expected_type: OperandType::INTEGER,
                                        });
                                    }
                                },
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
                self.emit_jump(ip, 1, &mut function_builder, &instruction_blocks)?;
            }
        }

        function_builder.switch_to_block(return_block);
        function_builder.append_block_param(return_block, I64);
        function_builder.append_block_params_for_function_returns(return_block);

        let return_value = function_builder.block_params(return_block)[0];

        function_builder.ins().return_(&[return_value]);
        function_builder.seal_all_blocks();

        self.module
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

        self.module.clear_context(&mut compilation_context);

        Ok(())
    }

    fn compile_stackless_function(
        &mut self,
        function_id: FuncId,
        chunk: &Chunk,
        is_main: bool,
    ) -> Result<(), JitError> {
        info!(
            "Compiling stackless function {}",
            chunk.name.as_ref().map_or("anonymous", |path| path.inner())
        );

        let mut function_builder_context = FunctionBuilderContext::new();
        let mut compilation_context = self.module.make_context();
        let pointer_type = self.module.isa().pointer_type();

        compilation_context
            .func
            .signature
            .params
            .push(AbiParam::new(pointer_type));
        compilation_context
            .func
            .signature
            .params
            .push(AbiParam::new(pointer_type));
        compilation_context
            .func
            .signature
            .params
            .push(AbiParam::new(pointer_type));
        compilation_context
            .func
            .signature
            .params
            .push(AbiParam::new(pointer_type));
        compilation_context
            .func
            .signature
            .params
            .push(AbiParam::new(pointer_type));

        let mut function_builder =
            FunctionBuilder::new(&mut compilation_context.func, &mut function_builder_context);

        #[cfg(debug_assertions)]
        let log_operation_function = {
            let mut log_operation_signature = Signature::new(self.module.isa().default_call_conv());

            log_operation_signature.params.push(AbiParam::new(I8));
            log_operation_signature.returns = vec![];

            self.declare_imported_function(
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
        let return_register_pointer = function_builder.block_params(function_entry_block)[3];
        let return_type_pointer = function_builder.block_params(function_entry_block)[4];

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
                let operation_code_instruction =
                    function_builder.ins().iconst(I8, operation.0 as i64);

                function_builder
                    .ins()
                    .call(log_operation_function, &[operation_code_instruction]);
            }

            match operation {
                Operation::LOAD => {
                    {
                        let Load {
                            destination,
                            operand,
                            r#type,
                            jump_next,
                        } = Load::from(*current_instruction);
                        let result_register = match r#type {
                            OperandType::INTEGER => self.get_integer(
                                operand,
                                current_frame_base_address,
                                chunk,
                                &mut function_builder,
                            )?,
                            _ => {
                                return Err(JitError::UnsupportedOperandType {
                                    operand_type: r#type,
                                });
                            }
                        };

                        self.set_register(
                            destination.index as usize,
                            result_register,
                            current_frame_base_address,
                            &mut function_builder,
                        )?;

                        if jump_next {
                            self.emit_jump(ip, 2, &mut function_builder, &[])?;
                        }

                        Ok(())
                    }?;
                }
                Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL => {
                    {
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
                                let left_value = self.get_integer(
                                    left,
                                    current_frame_base_address,
                                    chunk,
                                    &mut function_builder,
                                )?;
                                let right_value = self.get_integer(
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

                        Ok(())
                    }?;
                }
                Operation::ADD => {
                    {
                        let Add {
                            destination,
                            left,
                            right,
                            r#type,
                        } = Add::from(*current_instruction);
                        let result_register = match r#type {
                            OperandType::INTEGER => {
                                let left_value = self.get_integer(
                                    left,
                                    current_frame_base_address,
                                    chunk,
                                    &mut function_builder,
                                )?;
                                let right_value = self.get_integer(
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

                        self.set_register(
                            destination.index as usize,
                            result_register,
                            current_frame_base_address,
                            &mut function_builder,
                        )?;

                        Ok(())
                    }?;
                }
                Operation::SUBTRACT => {
                    {
                        let Subtract {
                            destination,
                            left,
                            right,
                            r#type,
                        } = Subtract::from(*current_instruction);
                        let result_register = match r#type {
                            OperandType::INTEGER => {
                                let left_value = self.get_integer(
                                    left,
                                    current_frame_base_address,
                                    chunk,
                                    &mut function_builder,
                                )?;
                                let right_value = self.get_integer(
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

                        self.set_register(
                            destination.index as usize,
                            result_register,
                            current_frame_base_address,
                            &mut function_builder,
                        )?;

                        Ok(())
                    }?;
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
                    let callee_function_ids = self.function_ids.get(prototype_index).ok_or(
                        JitError::FunctionIndexOutOfBounds {
                            ip,
                            function_index: prototype_index,
                            total_function_count: self.function_ids.len(),
                        },
                    )?;
                    let callee_function_reference = self
                        .module
                        .declare_func_in_func(callee_function_ids.direct, function_builder.func);

                    let call_arguments_list = chunk
                        .call_argument_lists
                        .get(arguments_index)
                        .ok_or(JitError::ArgumentsIndexOutOfBounds {
                            arguments_index,
                            total_argument_count: chunk.call_argument_lists.len(),
                        })?;
                    let mut arguments = Vec::with_capacity(call_arguments_list.len() + 3);

                    for (address, r#type) in call_arguments_list {
                        let argument_value = match *r#type {
                            OperandType::INTEGER => {
                                let integer_value = self.get_integer(
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

                    self.set_register(
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
                        self.emit_jump(
                            ip,
                            (offset + 1) as isize,
                            &mut function_builder,
                            &instruction_blocks,
                        )?;
                    } else {
                        self.emit_jump(
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
                            OperandType::INTEGER => {
                                let integer_value = self.get_integer(
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

                    let current_length = function_builder.ins().load(
                        I64,
                        MemFlags::new(),
                        call_stack_length_pointer,
                        0,
                    );
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
                self.emit_jump(ip, 1, &mut function_builder, &instruction_blocks)?;
            }
        }

        function_builder.switch_to_block(return_block);
        function_builder.ins().return_(&[]);
        function_builder.seal_all_blocks();

        self.module
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

        self.module.clear_context(&mut compilation_context);

        Ok(())
    }

    fn emit_jump(
        &self,
        ip: usize,
        jump_distance: isize,
        function_builder: &mut FunctionBuilder,
        instruction_blocks: &[Block],
    ) -> Result<(), JitError> {
        let target_ip = ip as isize + jump_distance;

        if target_ip < 0 {
            return Err(JitError::JumpTargetOutOfBounds {
                target_instruction_pointer: target_ip,
                total_instruction_count: instruction_blocks.len(),
            });
        }

        let target_ip = target_ip as usize;

        if target_ip >= instruction_blocks.len() {
            return Err(JitError::JumpTargetOutOfBounds {
                target_instruction_pointer: target_ip as isize,
                total_instruction_count: instruction_blocks.len(),
            });
        }

        function_builder
            .ins()
            .jump(instruction_blocks[target_ip], &[]);

        Ok(())
    }

    fn get_integer(
        &self,
        address: Address,
        frame_base_address: CraneliftValue,
        chunk: &Chunk,
        function_builder: &mut FunctionBuilder,
    ) -> Result<CraneliftValue, JitError> {
        let address_index = address.index as usize;
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
            MemoryKind::CONSTANT => match chunk.constants[address_index].as_integer() {
                Some(integer) => function_builder.ins().iconst(I64, integer),
                None => {
                    return Err(JitError::InvalidConstantType {
                        constant_index: address_index,
                        expected_type: OperandType::INTEGER,
                    });
                }
            },
            _ => {
                return Err(JitError::UnsupportedMemoryKind {
                    memory_kind: address.memory,
                });
            }
        };

        Ok(jit_value)
    }

    fn set_register(
        &self,
        register_index: usize,
        value: CraneliftValue,
        frame_base_address: CraneliftValue,
        function_builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let relative_index = function_builder.ins().iconst(I64, register_index as i64);
        let byte_offset = function_builder
            .ins()
            .imul_imm(relative_index, size_of::<Register>() as i64);
        let address = function_builder.ins().iadd(frame_base_address, byte_offset);

        function_builder
            .ins()
            .store(MemFlags::new(), value, address, 0);

        Ok(())
    }

    fn declare_imported_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
        name: &str,
        signature: Signature,
    ) -> Result<FuncRef, JitError> {
        let function_id = self
            .module
            .declare_function(name, Linkage::Import, &signature)
            .map_err(|error| JitError::CraneliftModuleError {
                message: format!("Failed to declare {name} function: {error}"),
            })?;
        let function_reference = self
            .module
            .declare_func_in_func(function_id, function_builder.func);

        Ok(function_reference)
    }
}

#[derive(Clone, Copy)]
struct FunctionIds {
    direct: FuncId,
    stackless: FuncId,
}

pub type JitLogic = fn(
    call_stack: *mut u8,
    call_stack_length: *mut usize,
    register_stack: *mut Register,
    return_register: *mut Register,
    return_type: *mut OperandType,
) -> ThreadStatus;
