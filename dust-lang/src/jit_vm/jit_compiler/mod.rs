mod functions;
mod jit_error;

use std::mem::transmute;

use functions::*;
pub use jit_error::{JIT_ERROR_TEXT, JitError};

use cranelift::{
    codegen::{
        CodegenError,
        ir::{BlockArg, FuncRef, InstBuilder},
    },
    frontend::Switch,
    prelude::{
        AbiParam, Block, EntityRef, FunctionBuilder, FunctionBuilderContext, IntCC, MemFlags,
        Signature, Value as CraneliftValue,
        types::{I8, I64},
    },
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module, ModuleError};
use tracing::{Level, info, trace};

use crate::{
    Address, Chunk, Instruction, OperandType, Operation, Program, Register, ThreadStatus,
    instruction::{Add, Call, Jump, Load, MemoryKind, Return},
    jit_vm::call_stack::{
        get_call_frame, get_frame_function_index, pop_call_frame, push_call_frame,
    },
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
        builder.symbol("log_value", log_value as *const u8);

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
        call_frame_registers_pointer: CraneliftValue,
        function_builder: &mut FunctionBuilder,
        ip: usize,
        instruction: Instruction,
    ) -> Result<CraneliftValue, JitError> {
        let jit_value = match address.memory {
            MemoryKind::REGISTER => {
                let register_byte_offset = (address.index * size_of::<Register>()) as i32;

                function_builder.ins().load(
                    I64,
                    MemFlags::new(),
                    call_frame_registers_pointer,
                    register_byte_offset,
                )
            }
            MemoryKind::CONSTANT => match chunk.constants[address.index].as_integer() {
                Some(integer) => function_builder.ins().iconst(I64, integer),
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

    pub fn compile(&mut self) -> Result<JitLogic, JitError> {
        let span = tracing::span!(Level::INFO, "JIT_Compiler");
        let _enter = span.enter();

        let loop_pointer = self.compile_loop()?;

        Ok(unsafe { transmute::<*const u8, JitLogic>(loop_pointer) })
    }

    fn compile_loop(&mut self) -> Result<*const u8, JitError> {
        let mut context = self.module.make_context();

        let main_function_reference = {
            let main_function_id = self.compile_chunk(&self.program.main_chunk)?;

            self.module
                .declare_func_in_func(main_function_id, &mut context.func)
        };
        let function_references = {
            let mut references = Vec::with_capacity(self.program.prototypes.len());

            for chunk in &self.program.prototypes {
                let function_id = self.compile_chunk(chunk)?;
                let function_reference = self
                    .module
                    .declare_func_in_func(function_id, &mut context.func);

                references.push(function_reference);
            }

            references
        };

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
        let _function_blocks = {
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
            let no_op_instruction = function_builder
                .ins()
                .iconst(I64, Instruction::no_op().0 as i64);
            let null_function_index = function_builder.ins().iconst(I64, u32::MAX as i64);

            push_call_frame(
                zero,
                zero,
                null_function_index,
                no_op_instruction,
                zero,
                register_count,
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
            for (block, function_reference) in _function_blocks
                .into_iter()
                .zip(function_references.into_iter())
            {
                function_builder.switch_to_block(block);
                function_builder.ins().call(
                    function_reference,
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

    pub fn compile_chunk(&mut self, chunk: &Chunk) -> Result<FuncId, JitError> {
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

        #[cfg(debug_assertions)]
        let _log_value_function = {
            let mut log_value_signature = Signature::new(self.module.isa().default_call_conv());

            log_value_signature.params.push(AbiParam::new(I64));
            log_value_signature.returns = vec![];

            self.declare_imported_function(&mut function_builder, "log_value", log_value_signature)?
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

        let early_return_block = function_builder.create_block();

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
            current_frame_function_index,
            current_frame_register_range_start,
            current_frame_register_range_end,
            current_frame_arguments_index,
        ) = pop_call_frame(
            call_stack_pointer,
            call_stack_length_pointer,
            &mut function_builder,
        );

        switch.emit(&mut function_builder, current_frame_ip, early_return_block);

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
                    let result_register = match r#type {
                        OperandType::INTEGER => self.get_integer(
                            operand,
                            chunk,
                            register_stack_pointer,
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
                        register_stack_pointer,
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
                                register_stack_pointer,
                                &mut function_builder,
                                ip,
                                *current_instruction,
                            )?;
                            let right_value = self.get_integer(
                                right,
                                chunk,
                                register_stack_pointer,
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
                                register_stack_pointer,
                                &mut function_builder,
                                ip,
                                *current_instruction,
                            )?;
                            let right_value = self.get_integer(
                                right,
                                chunk,
                                register_stack_pointer,
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
                        register_stack_pointer,
                        register_offset,
                    );
                }
                Operation::CALL => {
                    let Call {
                        destination,
                        prototype_index,
                        arguments_index,
                        return_type,
                    } = Call::from(*current_instruction);
                    let zero = function_builder.ins().iconst(I64, 0);
                    let function_index = function_builder.ins().iconst(I64, prototype_index as i64);
                    let arguments_index =
                        function_builder.ins().iconst(I64, arguments_index as i64);
                    let register_range_start = current_frame_register_range_end;
                    let register_range_end = function_builder
                        .ins()
                        .iadd_imm(register_range_start, chunk.register_tags.len() as i64);
                    let next_ip = function_builder.ins().iconst(I64, (ip + 1) as i64);

                    push_call_frame(
                        top_call_frame_index,
                        next_ip,
                        current_frame_function_index,
                        current_frame_register_range_start,
                        current_frame_register_range_end,
                        current_frame_arguments_index,
                        call_stack_pointer,
                        call_stack_length_pointer,
                        &mut function_builder,
                    );
                    push_call_frame(
                        call_stack_length,
                        zero,
                        function_index,
                        register_range_start,
                        register_range_end,
                        arguments_index,
                        call_stack_pointer,
                        call_stack_length_pointer,
                        &mut function_builder,
                    );

                    function_builder.ins().return_(&[]);
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
                    if should_return_value != 0 {
                        let (value_to_return, return_type) = match r#type {
                            OperandType::INTEGER => {
                                let integer_value = self.get_integer(
                                    return_value_address,
                                    chunk,
                                    register_stack_pointer,
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
                            return_register_pointer,
                            0,
                        );
                        function_builder.ins().store(
                            MemFlags::new(),
                            return_type,
                            return_type_pointer,
                            0,
                        );
                        function_builder.ins().return_(&[]);
                    } else {
                        function_builder.ins().store(
                            MemFlags::new(),
                            CraneliftValue::new(0),
                            return_register_pointer,
                            0,
                        );
                        function_builder.ins().return_(&[]);
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
                    | Operation::CALL
                    | Operation::JUMP
                    | Operation::RETURN
            ) {
                self.emit_jump(ip, 1, &mut function_builder, &instruction_blocks)?;
            }
        }

        trace!("{instruction_count} instruction(s) compiled successfully");

        function_builder.switch_to_block(early_return_block);
        function_builder.ins().return_(&[]);
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

        info!(
            "Finished compiling function {}",
            chunk.name.as_ref().map_or("anonymous", |path| path.inner()),
        );

        self.module.clear_context(&mut compilation_context);

        Ok(compiled_function_id)
    }
}

pub type JitLogic = fn(
    call_stack: *mut u8,
    call_stack_length: *mut usize,
    register_stack: *mut Register,
    return_register: *mut Register,
    return_type: *mut OperandType,
) -> ThreadStatus;
