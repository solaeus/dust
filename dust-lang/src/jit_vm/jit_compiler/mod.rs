mod functions;
mod jit_error;

use std::{
    iter,
    mem::{offset_of, transmute},
    u32,
};

use functions::*;
pub use jit_error::{JIT_ERROR_TEXT, JitError};

use cranelift::{
    codegen::{
        CodegenError,
        ir::{FuncRef, InstBuilder, immediates::Offset32},
    },
    frontend::Switch,
    prelude::{
        types::{I8, I32, I64},
        *,
    },
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module, ModuleError};
use tracing::{Level, info, trace};

use crate::{
    Address, Chunk, Instruction, OperandType, Operation, Program, Register, Thread, ThreadStatus,
    instruction::{Add, Jump, Load, MemoryKind, Return, Test},
};

pub struct JitCompiler<'a> {
    module: JITModule,
    program: &'a Program,
}

impl<'a> JitCompiler<'a> {
    pub fn new(program: &'a Program) -> Self {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();

        builder.symbol("concatenate_strings", concatenate_strings as *const u8);
        builder.symbol("log_operation", log_operation as *const u8);

        let module = JITModule::new(builder);

        Self { module, program }
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
                ip,
                target_instruction_pointer: target_ip,
                total_instruction_count: instruction_blocks.len(),
            });
        }

        let target_ip = target_ip as usize;

        if target_ip >= instruction_blocks.len() {
            return Err(JitError::JumpTargetOutOfBounds {
                ip,
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
        chunk: &Chunk,
        call_frame_registers_pointer: Value,
        function_builder: &mut FunctionBuilder,
        ip: usize,
        instruction: Instruction,
    ) -> Result<Value, JitError> {
        let jit_value = match address.memory {
            MemoryKind::REGISTER => {
                let register_byte_offset = (address.index * size_of::<Register>()) as i32;

                function_builder.ins().load(
                    types::I64,
                    MemFlags::new(),
                    call_frame_registers_pointer,
                    register_byte_offset,
                )
            }
            MemoryKind::CONSTANT => match chunk.constants[address.index].as_integer() {
                Some(integer) => function_builder.ins().iconst(types::I64, integer),
                None => {
                    return Err(JitError::InvalidConstantType {
                        ip,
                        instruction,
                        constant_index: address.index,
                        expected_type: OperandType::INTEGER,
                    });
                }
            },
            _ => {
                return Err(JitError::UnsupportedMemoryKind {
                    ip,
                    instruction,
                    memory_kind: address.memory,
                });
            }
        };

        Ok(jit_value)
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

    fn return_run_status(&mut self, function_builder: &mut FunctionBuilder, status: ThreadStatus) {
        let value = function_builder
            .ins()
            .iconst(ThreadStatus::CRANELIFT_TYPE, status as i64);

        function_builder.ins().return_(&[value]);
    }

    pub fn compile(&mut self) -> Result<JitLogic, JitError> {
        let span = tracing::span!(Level::INFO, "JIT");
        let _enter = span.enter();

        let trampoline_pointer = self.compile_loop()?;

        Ok(unsafe { transmute::<*const u8, JitLogic>(trampoline_pointer) })
    }

    pub fn compile_loop(&mut self) -> Result<*const u8, JitError> {
        let mut context = self.module.make_context();
        let all_chunks = self
            .program
            .prototypes
            .iter()
            .chain(iter::once(&self.program.main_chunk));
        let mut functions = Vec::with_capacity(self.program.prototypes.len() + 1);

        for chunk in all_chunks {
            let function_reference = self.compile_chunk(chunk)?;
            let function_reference = self
                .module
                .declare_func_in_func(function_reference, &mut context.func);

            functions.push(function_reference);
        }

        info!("Compiled main and {} functions", functions.len() - 1);

        let function_ids = functions.as_slice();
        let pointer_type = self.module.isa().pointer_type();

        context
            .func
            .signature
            .params
            .push(AbiParam::new(pointer_type));
        context
            .func
            .signature
            .params
            .push(AbiParam::new(pointer_type));
        context
            .func
            .signature
            .params
            .push(AbiParam::new(pointer_type));
        context
            .func
            .signature
            .params
            .push(AbiParam::new(pointer_type));
        context
            .func
            .signature
            .returns
            .push(AbiParam::new(ThreadStatus::CRANELIFT_TYPE));

        let function_id = self
            .module
            .declare_anonymous_function(&context.func.signature)
            .map_err(|error| JitError::CraneliftModuleError {
                message: error.to_string(),
            })?;
        let mut function_builder_context = FunctionBuilderContext::new();
        let mut function_builder =
            FunctionBuilder::new(&mut context.func, &mut function_builder_context);

        let entry_block = {
            let block = function_builder.create_block();

            function_builder.append_block_params_for_function_params(block);

            block
        };
        let main_block = {
            let block = function_builder.create_block();

            function_builder.append_block_params_for_function_params(block);

            block
        };
        let return_block = {
            let block = function_builder.create_block();

            function_builder.append_block_param(block, I32);

            block
        };

        {
            function_builder.switch_to_block(entry_block);

            let call_stack = function_builder.block_params(entry_block)[0];
            let register_stack = function_builder.block_params(entry_block)[1];
            let return_register = function_builder.block_params(entry_block)[2];
            let return_type = function_builder.block_params(entry_block)[3];

            function_builder.ins().jump(
                main_block,
                &[
                    call_stack.into(),
                    register_stack.into(),
                    return_register.into(),
                    return_type.into(),
                ],
            );
        }

        info!("Compiling execution loop");

        {
            function_builder.switch_to_block(main_block);

            let call_stack_length = Variable::new(0);
            let initial_length = function_builder.ins().iconst(I32, 0);

            function_builder.declare_var(I32);
            function_builder.def_var(call_stack_length, initial_length);

            let call_stack = function_builder.block_params(main_block)[0];
            let register_stack = function_builder.block_params(main_block)[1];
            let return_register = function_builder.block_params(main_block)[2];
            let return_type = function_builder.block_params(main_block)[3];
            let main_function = *function_ids.last().unwrap();
            let call_instruction = function_builder.ins().call(
                main_function,
                &[call_stack, register_stack, return_register, return_type],
            );
            let next_function_index = function_builder.inst_results(call_instruction)[0];
            let current_call_stack_length = function_builder.use_var(call_stack_length);
            let out_of_bounds = function_builder.ins().icmp(
                IntCC::UnsignedGreaterThanOrEqual,
                next_function_index,
                current_call_stack_length,
            );
            let return_thread_status = function_builder
                .ins()
                .iconst(ThreadStatus::CRANELIFT_TYPE, ThreadStatus::Return as i64);

            function_builder.ins().brif(
                out_of_bounds,
                return_block,
                &[return_thread_status.into()],
                main_block,
                &[
                    call_stack.into(),
                    register_stack.into(),
                    return_register.into(),
                    return_type.into(),
                ],
            );
        }

        info!("Execution loop compiled successfully");

        {
            function_builder.switch_to_block(return_block);

            let return_value = function_builder.block_params(return_block)[0];

            function_builder.ins().return_(&[return_value]);
        }

        function_builder.seal_all_blocks();
        function_builder.finalize();
        self.module
            .define_function(function_id, &mut context)
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

        Ok(self.module.get_finalized_function(function_id))
    }

    fn compile_chunk(&mut self, chunk: &Chunk) -> Result<FuncId, JitError> {
        info!(
            "Compiling function {}",
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
            .returns
            .push(AbiParam::new(ThreadStatus::CRANELIFT_TYPE));

        let mut function_builder =
            FunctionBuilder::new(&mut compilation_context.func, &mut function_builder_context);

        #[cfg(debug_assertions)]
        let log_operation_function = {
            let mut log_operation_signature = Signature::new(self.module.isa().default_call_conv());

            log_operation_signature
                .params
                .push(AbiParam::new(types::I8));
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
        let mut switch = Switch::new();

        for ip in 0..instruction_count {
            let block = function_builder.create_block();

            instruction_blocks.push(block);
            switch.set_entry(ip as u128, block);
        }

        let unreachable_final_block = function_builder.create_block();

        function_builder.switch_to_block(function_entry_block);
        function_builder.append_block_params_for_function_params(function_entry_block);

        let call_stack_pointer = function_builder.block_params(function_entry_block)[0];
        let call_stack_variable = function_builder.declare_var(pointer_type);

        let register_stack_pointer = function_builder.block_params(function_entry_block)[1];
        let register_stack_variable = function_builder.declare_var(pointer_type);

        let return_register_pointer = function_builder.block_params(function_entry_block)[2];
        let return_register_variable = function_builder.declare_var(pointer_type);

        let return_type_pointer = function_builder.block_params(function_entry_block)[3];
        let return_type_variable = function_builder.declare_var(pointer_type);

        function_builder.def_var(call_stack_variable, call_stack_pointer);
        function_builder.def_var(register_stack_variable, register_stack_pointer);
        function_builder.def_var(return_register_variable, return_register_pointer);
        function_builder.def_var(return_type_variable, return_type_pointer);

        let call_stack_variable = function_builder.use_var(call_stack_variable);
        let register_stack_variable = function_builder.use_var(register_stack_variable);
        let return_register_variable = function_builder.use_var(return_register_variable);
        let return_type_variable = function_builder.use_var(return_type_variable);

        let ip_offset = call_stack!(ip_offset, 0, call_stack_pointer);
        let ip_value = function_builder.ins().load(
            types::I64,
            MemFlags::new(),
            call_stack_pointer,
            Offset32::new(ip_offset as i32),
        );

        switch.emit(&mut function_builder, ip_value, unreachable_final_block);

        let instruction_count = instruction_blocks.len();

        for ip in 0..instruction_count {
            let current_instruction = &bytecode_instructions[ip];
            let operation = current_instruction.operation();
            let instruction_block = instruction_blocks[ip];

            function_builder.switch_to_block(instruction_block);

            info!("Compiling {operation} at IP {ip}");

            #[cfg(debug_assertions)]
            {
                let operation_code_instruction =
                    function_builder.ins().iconst(types::I8, operation.0 as i64);

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
                        OperandType::INTEGER => self.get_integer(
                            operand,
                            chunk,
                            register_stack_variable,
                            &mut function_builder,
                            ip,
                            *current_instruction,
                        )?,
                        _ => todo!(),
                    };
                    let register_offset = (destination.index * size_of::<Register>()) as i32;

                    function_builder.ins().store(
                        MemFlags::new(),
                        result_register,
                        register_stack_variable,
                        register_offset,
                    );

                    if jump_next != 0 {
                        self.emit_jump(ip, 2, &mut function_builder, &instruction_blocks)?;

                        continue;
                    }
                }
                Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL => {
                    let comparator = current_instruction.a_field();
                    let left = current_instruction.b_address();
                    let right = current_instruction.c_address();
                    let r#type = current_instruction.operand_type();
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
                                chunk,
                                register_stack_variable,
                                &mut function_builder,
                                ip,
                                *current_instruction,
                            )?;
                            let right_value = self.get_integer(
                                right,
                                chunk,
                                register_stack_variable,
                                &mut function_builder,
                                ip,
                                *current_instruction,
                            )?;

                            function_builder
                                .ins()
                                .icmp(comparison, left_value, right_value)
                        }
                        _ => todo!(),
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
                            let left_value = self.get_integer(
                                left,
                                chunk,
                                register_stack_variable,
                                &mut function_builder,
                                ip,
                                *current_instruction,
                            )?;
                            let right_value = self.get_integer(
                                right,
                                chunk,
                                register_stack_variable,
                                &mut function_builder,
                                ip,
                                *current_instruction,
                            )?;

                            function_builder.ins().iadd(left_value, right_value)
                        }
                        _ => todo!(),
                    };
                    let register_offset = (destination.index * size_of::<Register>()) as i32;

                    function_builder.ins().store(
                        MemFlags::new(),
                        result_register,
                        register_stack_variable,
                        register_offset,
                    );
                }
                Operation::JUMP => {
                    let Jump {
                        offset,
                        is_positive,
                    } = Jump::from(*current_instruction);

                    if is_positive != 0 {
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
                    let return_status_value = function_builder
                        .ins()
                        .iconst(ThreadStatus::CRANELIFT_TYPE, ThreadStatus::Return as i64);

                    if should_return_value != 0 {
                        let (value_to_return, return_type) = match r#type {
                            OperandType::INTEGER => {
                                let integer_value = self.get_integer(
                                    return_value_address,
                                    chunk,
                                    register_stack_variable,
                                    &mut function_builder,
                                    ip,
                                    *current_instruction,
                                )?;
                                let integer_type = function_builder
                                    .ins()
                                    .iconst(I8, OperandType::INTEGER.0 as i64);

                                (integer_value, integer_type)
                            }
                            _ => todo!(),
                        };

                        function_builder.ins().store(
                            MemFlags::new(),
                            value_to_return,
                            return_register_variable,
                            0,
                        );
                        function_builder.ins().store(
                            MemFlags::new(),
                            return_type,
                            return_type_variable,
                            0,
                        );
                        function_builder.ins().return_(&[return_status_value]);
                    } else {
                        function_builder.ins().store(
                            MemFlags::new(),
                            Value::new(0),
                            return_register_pointer,
                            0,
                        );
                        function_builder.ins().return_(&[return_status_value]);
                    }
                }
                _ => {
                    return Err(JitError::UnhandledOperation {
                        ip,
                        instruction: *current_instruction,
                        operation,
                    });
                }
            }

            if !matches!(
                operation,
                Operation::EQUAL
                    | Operation::LESS
                    | Operation::LESS_EQUAL
                    | Operation::JUMP
                    | Operation::RETURN
            ) {
                self.emit_jump(ip, 1, &mut function_builder, &instruction_blocks)?;
            }
        }

        trace!("{instruction_count} instruction(s) compiled successfully");

        function_builder.switch_to_block(unreachable_final_block);

        let return_status_value = function_builder
            .ins()
            .iconst(ThreadStatus::CRANELIFT_TYPE, ThreadStatus::Return as i64);

        function_builder.ins().return_(&[return_status_value]);
        function_builder.seal_all_blocks();

        let compiled_function_id = self
            .module
            .declare_anonymous_function(&compilation_context.func.signature)
            .map_err(|error| JitError::CraneliftModuleError {
                message: format!("Failed to declare chunk function: {error}"),
            })?;

        self.module
            .define_function(compiled_function_id, &mut compilation_context)
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
        self.module.clear_context(&mut compilation_context);

        info!(
            "Finished compiling function {}",
            chunk.name.as_ref().map_or("anonymous", |path| path.inner())
        );

        Ok(compiled_function_id)
    }
}

pub type JitLogic = fn(
    call_stack: *mut u8,
    register_stack: *mut Register,
    return_register: *mut Register,
    return_type: *mut OperandType,
) -> ThreadStatus;
