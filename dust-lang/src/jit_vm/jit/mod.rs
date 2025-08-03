mod functions;
mod jit_error;

use std::mem::offset_of;

use functions::*;
pub use jit_error::{JIT_ERROR_TEXT, JitError};

use cranelift::{codegen::ir::FuncRef, frontend::Switch, prelude::*};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use tracing::{Level, info};

use crate::{
    Address, CallFrame, Chunk, Instruction, Object, OperandType, Operation, Register, Thread,
    ThreadStatus,
    instruction::{Jump, Load, MemoryKind, Return, Test},
    jit_vm::ObjectPool,
};

const STATUS_TYPE: types::Type = match size_of::<ThreadStatus>() {
    4 => types::I32,
    _ => types::I64,
};

pub struct Jit<'a> {
    module: JITModule,
    chunk: &'a Chunk,
    object_pool: &'a mut ObjectPool,
}

impl<'a> Jit<'a> {
    pub fn new(chunk: &'a Chunk, object_pool: &'a mut ObjectPool) -> Self {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();

        builder.symbol("concatenate_strings", concatenate_strings as *const u8);
        builder.symbol("log_operation", log_operation as *const u8);

        let module = JITModule::new(builder);

        Self {
            module,
            chunk,
            object_pool,
        }
    }

    fn terminate_with_jump(
        &self,
        function_builder: &mut FunctionBuilder,
        ip: usize,
        instruction_blocks: &[Block],
    ) -> Result<(), JitError> {
        let next_ip = ip + 1;

        if next_ip >= instruction_blocks.len() {
            return Err(JitError::BranchTargetOutOfBounds {
                ip,
                branch_target_instruction_pointer: next_ip,
                total_instruction_count: instruction_blocks.len(),
            });
        }

        function_builder
            .ins()
            .jump(instruction_blocks[next_ip], &[]);

        Ok(())
    }

    fn get_integer(
        &self,
        address: Address,
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
            MemoryKind::CONSTANT => match self.chunk.constants[address.index].as_integer() {
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

    fn get_string(
        &mut self,
        address: Address,
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
            MemoryKind::CONSTANT => match self.chunk.constants[address.index].as_string() {
                Some(string) => {
                    let object = Object::string(string.clone());
                    let key = self.object_pool.allocate(object);

                    function_builder.ins().iconst(types::I64, key as i64)
                }
                None => {
                    return Err(JitError::InvalidConstantType {
                        ip,
                        instruction,
                        constant_index: address.index,
                        expected_type: OperandType::STRING,
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
        let value = function_builder.ins().iconst(STATUS_TYPE, status as i64);

        function_builder.ins().return_(&[value]);
    }

    pub fn compile(&mut self) -> Result<JitChunk, JitError> {
        let span = tracing::span!(Level::INFO, "JIT");
        let _enter = span.enter();

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
            .returns
            .push(AbiParam::new(STATUS_TYPE));

        let mut function_builder =
            FunctionBuilder::new(&mut compilation_context.func, &mut function_builder_context);
        let mut concatenate_strings_signature =
            Signature::new(self.module.isa().default_call_conv());

        concatenate_strings_signature
            .params
            .push(AbiParam::new(pointer_type));
        concatenate_strings_signature
            .params
            .push(AbiParam::new(pointer_type));
        concatenate_strings_signature
            .params
            .push(AbiParam::new(pointer_type));
        concatenate_strings_signature
            .returns
            .push(AbiParam::new(pointer_type));

        let concatenate_strings_function = self.declare_imported_function(
            &mut function_builder,
            "concatenate_strings",
            concatenate_strings_signature,
        )?;
        let mut log_operation_signature = Signature::new(self.module.isa().default_call_conv());

        log_operation_signature
            .params
            .push(AbiParam::new(types::I8));
        log_operation_signature.returns = vec![];

        let log_operation_function = self.declare_imported_function(
            &mut function_builder,
            "log_operation",
            log_operation_signature,
        )?;

        let bytecode_instructions = &self.chunk.instructions;
        let instruction_count = bytecode_instructions.len();
        let mut instruction_blocks = Vec::with_capacity(instruction_count);
        let mut switch = Switch::new();

        for ip in 0..instruction_count {
            let block = function_builder.create_block();

            instruction_blocks.push(block);
            switch.set_entry(ip as u128, block);
        }

        let function_entry_block = function_builder.create_block();
        let unreachable_final_block = function_builder.create_block();

        function_builder.switch_to_block(function_entry_block);
        function_builder.append_block_params_for_function_params(function_entry_block);

        let variable_0 = function_builder.declare_var(pointer_type);
        let variable_1 = function_builder.declare_var(pointer_type);
        let variable_2 = function_builder.declare_var(pointer_type);

        let thread_runner_pointer = function_builder.block_params(function_entry_block)[0];
        let call_frame_pointer = function_builder.block_params(function_entry_block)[1];
        let registers_pointer = function_builder.block_params(function_entry_block)[2];

        function_builder.def_var(variable_0, thread_runner_pointer);
        function_builder.def_var(variable_1, call_frame_pointer);
        function_builder.def_var(variable_2, registers_pointer);

        let ip_offset = offset_of!(CallFrame, ip) as i32;
        let ip_value =
            function_builder
                .ins()
                .load(types::I64, MemFlags::new(), call_frame_pointer, ip_offset);

        switch.emit(&mut function_builder, ip_value, unreachable_final_block);

        for ip in 0..instruction_count {
            let current_instruction = &bytecode_instructions[ip];
            let operation = current_instruction.operation();

            info!("Compiling {operation} at IP {ip}");

            function_builder.switch_to_block(instruction_blocks[ip]);

            #[cfg(debug_assertions)]
            {
                let operation_code_instruction =
                    function_builder.ins().iconst(types::I8, operation.0 as i64);

                function_builder
                    .ins()
                    .call(log_operation_function, &[operation_code_instruction]);
            }

            let _thread_runner_pointer = function_builder.use_var(variable_0);
            let _call_frame_pointer = function_builder.use_var(variable_1);
            let registers_pointer = function_builder.use_var(variable_2);

            match operation {
                Operation::LOAD => {
                    let Load {
                        destination,
                        operand,
                        r#type,
                        ..
                    } = Load::from(*current_instruction);
                    let value = match r#type {
                        OperandType::INTEGER => self.get_integer(
                            operand,
                            registers_pointer,
                            &mut function_builder,
                            ip,
                            *current_instruction,
                        )?,
                        // OperandType::FLOAT => self.get_float(...)?,
                        OperandType::STRING => self.get_string(
                            operand,
                            registers_pointer,
                            &mut function_builder,
                            ip,
                            *current_instruction,
                        )?,
                        _ => {
                            return Err(JitError::UnsupportedOperandType {
                                ip,
                                instruction: *current_instruction,
                                operand_type: r#type,
                            });
                        }
                    };
                    let destination_register_byte_offset =
                        (destination.index * size_of::<Register>()) as i32;

                    function_builder.ins().store(
                        MemFlags::new(),
                        value,
                        registers_pointer,
                        destination_register_byte_offset,
                    );
                    self.terminate_with_jump(&mut function_builder, ip, &instruction_blocks)?;
                }
                Operation::ADD
                | Operation::SUBTRACT
                | Operation::MULTIPLY
                | Operation::DIVIDE
                | Operation::MODULO => {
                    let destination = current_instruction.destination();
                    let left = current_instruction.b_address();
                    let right = current_instruction.c_address();
                    let r#type = current_instruction.operand_type();
                    let result_value = match r#type {
                        OperandType::INTEGER => {
                            let left_integer = self.get_integer(
                                left,
                                registers_pointer,
                                &mut function_builder,
                                ip,
                                *current_instruction,
                            )?;
                            let right_integer = self.get_integer(
                                right,
                                registers_pointer,
                                &mut function_builder,
                                ip,
                                *current_instruction,
                            )?;

                            match current_instruction.operation() {
                                Operation::ADD => {
                                    function_builder.ins().iadd(left_integer, right_integer)
                                }
                                Operation::SUBTRACT => {
                                    function_builder.ins().isub(left_integer, right_integer)
                                }
                                Operation::MULTIPLY => {
                                    function_builder.ins().imul(left_integer, right_integer)
                                }
                                Operation::DIVIDE => {
                                    function_builder.ins().udiv(left_integer, right_integer)
                                }
                                Operation::MODULO => {
                                    function_builder.ins().urem(left_integer, right_integer)
                                }
                                _ => {
                                    return Err(JitError::UnhandledOperation {
                                        ip,
                                        instruction: *current_instruction,
                                        operation,
                                    });
                                }
                            }
                        }
                        OperandType::STRING => {
                            let left_string = self.get_string(
                                left,
                                registers_pointer,
                                &mut function_builder,
                                ip,
                                *current_instruction,
                            )?;
                            let right_string = self.get_string(
                                right,
                                registers_pointer,
                                &mut function_builder,
                                ip,
                                *current_instruction,
                            )?;
                            let concatenated_string_result = match current_instruction.operation() {
                                Operation::ADD => function_builder.ins().call(
                                    concatenate_strings_function,
                                    &[thread_runner_pointer, left_string, right_string],
                                ),
                                _ => {
                                    return Err(JitError::UnhandledOperation {
                                        ip,
                                        instruction: *current_instruction,
                                        operation,
                                    });
                                }
                            };

                            function_builder.inst_results(concatenated_string_result)[0]
                        }
                        _ => {
                            return Err(JitError::UnsupportedOperandType {
                                ip,
                                instruction: *current_instruction,
                                operand_type: r#type,
                            });
                        }
                    };

                    let destination_register_byte_offset =
                        (destination.index * size_of::<Register>()) as i32;

                    function_builder.ins().store(
                        MemFlags::new(),
                        result_value,
                        registers_pointer,
                        destination_register_byte_offset,
                    );
                    self.terminate_with_jump(&mut function_builder, ip, &instruction_blocks)?;
                }
                Operation::LESS | Operation::EQUAL | Operation::LESS_EQUAL => {
                    let comparator = current_instruction.destination().index != 0;
                    let left = current_instruction.b_address();
                    let right = current_instruction.c_address();
                    let r#type = current_instruction.operand_type();
                    let comparison_operation = match (operation, comparator) {
                        (Operation::LESS, true) => IntCC::SignedLessThan,
                        (Operation::LESS, false) => IntCC::SignedGreaterThanOrEqual,
                        (Operation::EQUAL, true) => IntCC::Equal,
                        (Operation::EQUAL, false) => IntCC::NotEqual,
                        (Operation::LESS_EQUAL, true) => IntCC::SignedLessThanOrEqual,
                        (Operation::LESS_EQUAL, false) => IntCC::SignedGreaterThan,
                        _ => unreachable!(),
                    };
                    let (left, right) = match r#type {
                        OperandType::INTEGER => (
                            self.get_integer(
                                left,
                                registers_pointer,
                                &mut function_builder,
                                ip,
                                *current_instruction,
                            )?,
                            self.get_integer(
                                right,
                                registers_pointer,
                                &mut function_builder,
                                ip,
                                *current_instruction,
                            )?,
                        ),
                        _ => {
                            return Err(JitError::UnsupportedOperandType {
                                ip,
                                instruction: *current_instruction,
                                operand_type: r#type,
                            });
                        }
                    };
                    let comparison_result =
                        function_builder
                            .ins()
                            .icmp(comparison_operation, left, right);

                    function_builder.ins().brif(
                        comparison_result,
                        instruction_blocks[ip + 2],
                        &[],
                        instruction_blocks[ip + 1],
                        &[],
                    );
                }
                Operation::TEST => {
                    let Test {
                        comparator,
                        operand,
                    } = Test::from(*current_instruction);
                    let operand_value = match operand.memory {
                        MemoryKind::REGISTER => {
                            let operand_byte_offset =
                                (operand.index * size_of::<Register>()) as i32;
                            function_builder.ins().load(
                                types::I64,
                                MemFlags::new(),
                                registers_pointer,
                                operand_byte_offset,
                            )
                        }
                        _ => {
                            return Err(JitError::UnsupportedMemoryKind {
                                ip,
                                instruction: *current_instruction,
                                memory_kind: operand.memory,
                            });
                        }
                    };
                    let condition = if comparator {
                        function_builder
                            .ins()
                            .icmp_imm(IntCC::NotEqual, operand_value, 0)
                    } else {
                        function_builder
                            .ins()
                            .icmp_imm(IntCC::Equal, operand_value, 0)
                    };
                    let skip_next_instruction_pointer = ip + 2;
                    let proceed_to_next_instruction_pointer = ip + 1;
                    let skip_next_instruction_block =
                        if skip_next_instruction_pointer < instruction_blocks.len() {
                            instruction_blocks[skip_next_instruction_pointer]
                        } else {
                            return Err(JitError::BranchTargetOutOfBounds {
                                ip,
                                branch_target_instruction_pointer: skip_next_instruction_pointer,
                                total_instruction_count: instruction_blocks.len(),
                            });
                        };
                    let proceed_to_next_instruction_block = if proceed_to_next_instruction_pointer
                        < instruction_blocks.len()
                    {
                        instruction_blocks[proceed_to_next_instruction_pointer]
                    } else {
                        return Err(JitError::BranchTargetOutOfBounds {
                            ip,
                            branch_target_instruction_pointer: proceed_to_next_instruction_pointer,
                            total_instruction_count: instruction_blocks.len(),
                        });
                    };

                    function_builder.ins().brif(
                        condition,
                        proceed_to_next_instruction_block,
                        &[],
                        skip_next_instruction_block,
                        &[],
                    );
                }
                Operation::CALL => {
                    let ip_offset = offset_of!(CallFrame, ip) as i32;
                    let next_call_offset = offset_of!(CallFrame, next_call_instruction) as i32;
                    let next_ip = function_builder.ins().iconst(types::I64, (ip + 1) as i64);

                    function_builder.ins().store(
                        MemFlags::new(),
                        next_ip,
                        call_frame_pointer,
                        ip_offset,
                    );

                    let next_call_value = function_builder
                        .ins()
                        .iconst(types::I64, current_instruction.0 as i64);

                    function_builder.ins().store(
                        MemFlags::new(),
                        next_call_value,
                        call_frame_pointer,
                        next_call_offset,
                    );
                    self.return_run_status(&mut function_builder, ThreadStatus::Call);
                }
                Operation::JUMP => {
                    let Jump {
                        offset,
                        is_positive,
                    } = Jump::from(*current_instruction);
                    let jump_target_ip = if is_positive != 0 {
                        ip + offset + 1
                    } else {
                        ip - offset
                    };

                    if jump_target_ip >= instruction_blocks.len() {
                        return Err(JitError::JumpTargetOutOfBounds {
                            ip,
                            target_instruction_pointer: jump_target_ip,
                            total_instruction_count: instruction_blocks.len(),
                        });
                    }

                    if jump_target_ip == ip {
                        return Err(JitError::JumpToSelf { ip });
                    }

                    function_builder
                        .ins()
                        .jump(instruction_blocks[jump_target_ip], &[]);
                }
                Operation::RETURN => {
                    let Return {
                        should_return_value,
                        return_value_address,
                        r#type,
                    } = Return::from(*current_instruction);

                    if should_return_value != 0 {
                        match r#type {
                            OperandType::INTEGER => match return_value_address.memory {
                                MemoryKind::REGISTER => {
                                    let register_byte_offset =
                                        (return_value_address.index * size_of::<Register>()) as i32;
                                    let register = function_builder.ins().load(
                                        types::I64,
                                        MemFlags::new(),
                                        registers_pointer,
                                        register_byte_offset,
                                    );
                                    let thread_return_value_offset =
                                        offset_of!(Thread, return_value) as i32;

                                    function_builder.ins().store(
                                        MemFlags::new(),
                                        register,
                                        thread_runner_pointer,
                                        thread_return_value_offset,
                                    );
                                }
                                MemoryKind::CONSTANT => {
                                    let integer = self.chunk.constants[return_value_address.index]
                                        .as_integer()
                                        .ok_or(JitError::InvalidConstantType {
                                            ip,
                                            instruction: *current_instruction,
                                            constant_index: return_value_address.index,
                                            expected_type: OperandType::INTEGER,
                                        })?;
                                    let thread_return_value_offset =
                                        offset_of!(Thread, return_value) as i32;
                                    let value = function_builder.ins().iconst(types::I64, integer);

                                    function_builder.ins().store(
                                        MemFlags::new(),
                                        value,
                                        thread_runner_pointer,
                                        thread_return_value_offset,
                                    );
                                }
                                _ => {
                                    return Err(JitError::UnsupportedMemoryKind {
                                        ip,
                                        instruction: *current_instruction,
                                        memory_kind: return_value_address.memory,
                                    });
                                }
                            },
                            _ => {
                                return Err(JitError::UnsupportedOperandType {
                                    ip,
                                    instruction: *current_instruction,
                                    operand_type: r#type,
                                });
                            }
                        }
                    }

                    self.return_run_status(&mut function_builder, ThreadStatus::Return);
                }
                _ => {
                    return Err(JitError::UnhandledOperation {
                        ip,
                        instruction: *current_instruction,
                        operation,
                    });
                }
            }
        }

        function_builder.switch_to_block(unreachable_final_block);
        self.return_run_status(&mut function_builder, ThreadStatus::Return);
        function_builder.seal_all_blocks();

        let compiled_function_id = self
            .module
            .declare_anonymous_function(&compilation_context.func.signature)
            .map_err(|error| JitError::CraneliftModuleError {
                message: format!("Failed to declare chunk function: {error}"),
            })?;

        self.module
            .define_function(compiled_function_id, &mut compilation_context)
            .map_err(|_| JitError::FunctionCompilationError {
                message: format!(
                    "Failed to define function:\n{}",
                    compilation_context.func.display()
                ),
                cranelift_ir: compilation_context.func.display().to_string(),
            })?;
        self.module.clear_context(&mut compilation_context);
        self.module
            .finalize_definitions()
            .map_err(|error| JitError::CraneliftModuleError {
                message: format!("Failed to finalize definitions: {error}"),
            })?;

        let compiled_function_pointer = self.module.get_finalized_function(compiled_function_id);
        let logic = unsafe {
            std::mem::transmute::<
                *const u8,
                extern "C" fn(*mut Thread, *mut CallFrame, *mut Register) -> ThreadStatus,
            >(compiled_function_pointer)
        };

        Ok(JitChunk {
            logic,
            constants: self.chunk.constants.clone(),
            argument_lists: self.chunk.argument_lists.clone(),
            register_tags: self.chunk.register_tags.clone(),
            return_type: self.chunk.r#type.return_type.as_operand_type(),
        })
    }
}

pub struct JitChunk {
    pub logic: extern "C" fn(*mut Thread, *mut CallFrame, *mut Register) -> ThreadStatus,
    pub constants: Vec<crate::Value>,
    pub argument_lists: Vec<Vec<(Address, OperandType)>>,
    pub register_tags: Vec<OperandType>,
    pub return_type: OperandType,
}
