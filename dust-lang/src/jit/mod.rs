use std::{collections::VecDeque, mem::offset_of};

mod jit_error;

pub use jit_error::JitError;

use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

use crate::{
    CallFrame, OperandType, Operation, Register, StrippedChunk, ThreadRunner,
    instruction::MemoryKind,
    instruction::{Add, Jump, Less, Load, Return},
};

pub extern "C" fn load_constant(value: u64) -> u64 {
    value
}

pub struct Jit {
    module: JITModule,
}

/// # Safety
/// This function dereferences a raw pointer and must only be called with a valid ThreadRunner pointer.
pub unsafe extern "C" fn set_return_value_integer(
    thread_runner: *mut ThreadRunner,
    integer_value: i64,
) {
    unsafe {
        (*thread_runner).return_value = Some(crate::Value::Integer(integer_value));
    }
}

impl Jit {
    pub fn new() -> Self {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
        builder.symbol(
            "set_return_value_integer",
            set_return_value_integer as *const u8,
        );

        let module = JITModule::new(builder);

        Self { module }
    }

    pub fn compile(
        &mut self,
        chunk: &StrippedChunk,
    ) -> Result<extern "C" fn(*mut ThreadRunner, *mut CallFrame), JitError> {
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

        let mut return_value_signature = compilation_context.func.signature.clone();
        let mut function_builder =
            FunctionBuilder::new(&mut compilation_context.func, &mut function_builder_context);

        let bytecode_instructions = &chunk.instructions;
        let constant_pool = &chunk.constants;
        let total_instruction_count = bytecode_instructions.len();

        let mut instruction_blocks = Vec::with_capacity(total_instruction_count);
        for _ in 0..total_instruction_count {
            instruction_blocks.push(function_builder.create_block());
        }
        let unreachable_final_block = function_builder.create_block();

        return_value_signature.params =
            vec![AbiParam::new(pointer_type), AbiParam::new(types::I64)];
        return_value_signature.returns = vec![];
        let mut return_value_signature = Signature::new(self.module.isa().default_call_conv());
        return_value_signature
            .params
            .push(AbiParam::new(pointer_type));
        return_value_signature
            .params
            .push(AbiParam::new(types::I64));
        return_value_signature.returns = vec![];
        let set_return_function_id = self
            .module
            .declare_function(
                "set_return_value_integer",
                Linkage::Import,
                &return_value_signature,
            )
            .map_err(|error| JitError::CraneliftModuleError {
                instruction_pointer: None,
                message: format!("Failed to declare set_return_value_integer function: {error}"),
            })?;
        let set_return_function_reference = self
            .module
            .declare_func_in_func(set_return_function_id, function_builder.func);

        let function_entry_block = function_builder.create_block();
        function_builder.switch_to_block(function_entry_block);
        function_builder.append_block_params_for_function_params(function_entry_block);
        function_builder.declare_var(Variable::new(0), pointer_type);
        function_builder.declare_var(Variable::new(1), pointer_type);
        let thread_runner_pointer = function_builder.block_params(function_entry_block)[0];
        let call_frame_pointer = function_builder.block_params(function_entry_block)[1];
        function_builder.def_var(Variable::new(0), thread_runner_pointer);
        function_builder.def_var(Variable::new(1), call_frame_pointer);

        function_builder.ins().jump(instruction_blocks[0], &[]);

        let mut processed_instructions = vec![false; total_instruction_count];
        let mut instruction_worklist = VecDeque::new();
        instruction_worklist.push_back(0);
        instruction_worklist.push_back(bytecode_instructions.len() - 1);

        while let Some(current_instruction_pointer) = instruction_worklist.pop_front() {
            if processed_instructions[current_instruction_pointer] {
                continue;
            }
            function_builder.switch_to_block(instruction_blocks[current_instruction_pointer]);
            let _thread_runner_pointer = function_builder.use_var(Variable::new(0));
            let call_frame_pointer = function_builder.use_var(Variable::new(1));
            let current_instruction = &bytecode_instructions[current_instruction_pointer];

            match current_instruction.operation() {
                Operation::LOAD => {
                    let Load {
                        destination,
                        operand,
                        r#type,
                        ..
                    } = Load::from(*current_instruction);
                    match operand.memory {
                        MemoryKind::REGISTER => {
                            let call_frame_registers_field_offset =
                                offset_of!(CallFrame, registers) as i32;
                            let call_frame_registers_pointer = function_builder.ins().load(
                                pointer_type,
                                MemFlags::new(),
                                call_frame_pointer,
                                call_frame_registers_field_offset,
                            );
                            let source_operand_index = function_builder
                                .ins()
                                .iconst(types::I64, operand.index as i64);
                            let source_register_byte_offset = function_builder.ins().imul_imm(
                                source_operand_index,
                                std::mem::size_of::<Register>() as i64,
                            );
                            let destination_operand_index = function_builder
                                .ins()
                                .iconst(types::I64, destination.index as i64);
                            let destination_register_byte_offset = function_builder.ins().imul_imm(
                                destination_operand_index,
                                std::mem::size_of::<Register>() as i64,
                            );
                            let source_register_address = function_builder
                                .ins()
                                .iadd(call_frame_registers_pointer, source_register_byte_offset);
                            let destination_register_address = function_builder.ins().iadd(
                                call_frame_registers_pointer,
                                destination_register_byte_offset,
                            );
                            let source_register_value = function_builder.ins().load(
                                types::I64,
                                MemFlags::new(),
                                source_register_address,
                                0,
                            );
                            function_builder.ins().store(
                                MemFlags::new(),
                                source_register_value,
                                destination_register_address,
                                0,
                            );
                        }
                        MemoryKind::CONSTANT => match r#type {
                            OperandType::INTEGER => {
                                let integer_constant_value =
                                    match constant_pool[operand.index].as_integer() {
                                        Some(value) => value,
                                        None => {
                                            return Err(JitError::InvalidConstantType {
                                                instruction_pointer: current_instruction_pointer,
                                                constant_index: operand.index,
                                                expected_type: OperandType::INTEGER,
                                                operation: "LOAD".to_string(),
                                            });
                                        }
                                    };
                                let cranelift_constant_value = function_builder
                                    .ins()
                                    .iconst(types::I64, integer_constant_value);
                                let call_frame_registers_field_offset =
                                    offset_of!(CallFrame, registers) as i32;
                                let call_frame_registers_pointer = function_builder.ins().load(
                                    pointer_type,
                                    MemFlags::new(),
                                    call_frame_pointer,
                                    call_frame_registers_field_offset,
                                );
                                let destination_operand_index = function_builder
                                    .ins()
                                    .iconst(types::I64, destination.index as i64);
                                let destination_register_byte_offset =
                                    function_builder.ins().imul_imm(
                                        destination_operand_index,
                                        std::mem::size_of::<Register>() as i64,
                                    );
                                let destination_register_address = function_builder.ins().iadd(
                                    call_frame_registers_pointer,
                                    destination_register_byte_offset,
                                );
                                function_builder.ins().store(
                                    MemFlags::new(),
                                    cranelift_constant_value,
                                    destination_register_address,
                                    0,
                                );
                            }
                            _ => {
                                return Err(JitError::UnsupportedOperandType {
                                    instruction_pointer: current_instruction_pointer,
                                    operand_type: r#type,
                                    operation: "LOAD".to_string(),
                                });
                            }
                        },
                        _ => {
                            return Err(JitError::UnsupportedMemoryKind {
                                instruction_pointer: current_instruction_pointer,
                                operation: "LOAD".to_string(),
                                memory_kind_description: format!("{:?}", operand.memory),
                            });
                        }
                    }
                }
                Operation::ADD => {
                    let Add {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Add::from(*current_instruction);
                    match r#type {
                        OperandType::INTEGER => {
                            let call_frame_registers_field_offset =
                                offset_of!(CallFrame, registers) as i32;
                            let call_frame_registers_pointer = function_builder.ins().load(
                                pointer_type,
                                MemFlags::new(),
                                call_frame_pointer,
                                call_frame_registers_field_offset,
                            );
                            let left_operand_index =
                                function_builder.ins().iconst(types::I64, left.index as i64);
                            let left_register_byte_offset = function_builder.ins().imul_imm(
                                left_operand_index,
                                std::mem::size_of::<Register>() as i64,
                            );
                            let left_register_address = function_builder
                                .ins()
                                .iadd(call_frame_registers_pointer, left_register_byte_offset);
                            let left_operand_value = function_builder.ins().load(
                                types::I64,
                                MemFlags::new(),
                                left_register_address,
                                0,
                            );
                            let right_operand_value = match right.memory {
                                MemoryKind::REGISTER => {
                                    let right_operand_index = function_builder
                                        .ins()
                                        .iconst(types::I64, right.index as i64);
                                    let right_register_byte_offset =
                                        function_builder.ins().imul_imm(
                                            right_operand_index,
                                            std::mem::size_of::<Register>() as i64,
                                        );
                                    let right_register_address = function_builder.ins().iadd(
                                        call_frame_registers_pointer,
                                        right_register_byte_offset,
                                    );
                                    function_builder.ins().load(
                                        types::I64,
                                        MemFlags::new(),
                                        right_register_address,
                                        0,
                                    )
                                }
                                MemoryKind::CONSTANT => {
                                    let integer_constant_value = match constant_pool[right.index]
                                        .as_integer()
                                    {
                                        Some(val) => val,
                                        None => {
                                            return Err(JitError::InvalidConstantType {
                                                instruction_pointer: current_instruction_pointer,
                                                constant_index: right.index,
                                                expected_type: OperandType::INTEGER,
                                                operation: "ADD".to_string(),
                                            });
                                        }
                                    };
                                    function_builder
                                        .ins()
                                        .iconst(types::I64, integer_constant_value)
                                }
                                _ => {
                                    return Err(JitError::UnsupportedMemoryKind {
                                        instruction_pointer: current_instruction_pointer,
                                        operation: "ADD".to_string(),
                                        memory_kind_description: format!("{:?}", right.memory),
                                    });
                                }
                            };
                            let addition_result_value = function_builder
                                .ins()
                                .iadd(left_operand_value, right_operand_value);
                            let destination_operand_index = function_builder
                                .ins()
                                .iconst(types::I64, destination.index as i64);
                            let destination_register_byte_offset = function_builder.ins().imul_imm(
                                destination_operand_index,
                                std::mem::size_of::<Register>() as i64,
                            );
                            let destination_register_address = function_builder.ins().iadd(
                                call_frame_registers_pointer,
                                destination_register_byte_offset,
                            );
                            function_builder.ins().store(
                                MemFlags::new(),
                                addition_result_value,
                                destination_register_address,
                                0,
                            );
                        }
                        _ => {
                            return Err(JitError::UnsupportedOperandType {
                                instruction_pointer: current_instruction_pointer,
                                operand_type: r#type,
                                operation: "ADD".to_string(),
                            });
                        }
                    }
                }
                Operation::LESS => {
                    let Less {
                        comparator,
                        left,
                        right,
                        r#type,
                    } = Less::from(*current_instruction);
                    match r#type {
                        OperandType::INTEGER => match (left.memory, right.memory) {
                            (MemoryKind::REGISTER, MemoryKind::CONSTANT) => {
                                let integer_constant_value =
                                    match constant_pool[right.index].as_integer() {
                                        Some(val) => val,
                                        None => {
                                            return Err(JitError::InvalidConstantType {
                                                instruction_pointer: current_instruction_pointer,
                                                constant_index: right.index,
                                                expected_type: OperandType::INTEGER,
                                                operation: "LESS".to_string(),
                                            });
                                        }
                                    };
                                let call_frame_registers_field_offset =
                                    offset_of!(CallFrame, registers) as i32;
                                let call_frame_registers_pointer = function_builder.ins().load(
                                    pointer_type,
                                    MemFlags::new(),
                                    call_frame_pointer,
                                    call_frame_registers_field_offset,
                                );
                                let left_register_byte_offset = function_builder.ins().iconst(
                                    pointer_type,
                                    (left.index * std::mem::size_of::<Register>()) as i64,
                                );
                                let left_register_address = function_builder
                                    .ins()
                                    .iadd(call_frame_registers_pointer, left_register_byte_offset);
                                let left_operand_value = function_builder.ins().load(
                                    types::I64,
                                    MemFlags::new(),
                                    left_register_address,
                                    0,
                                );
                                let cranelift_constant_value = function_builder
                                    .ins()
                                    .iconst(types::I64, integer_constant_value);
                                let comparison_result = if comparator != 0 {
                                    function_builder.ins().icmp(
                                        IntCC::SignedLessThan,
                                        left_operand_value,
                                        cranelift_constant_value,
                                    )
                                } else {
                                    function_builder.ins().icmp(
                                        IntCC::SignedGreaterThanOrEqual,
                                        left_operand_value,
                                        cranelift_constant_value,
                                    )
                                };

                                let skip_next_instruction_pointer = current_instruction_pointer + 2;
                                let proceed_to_next_instruction_pointer =
                                    current_instruction_pointer + 1;
                                let skip_next_instruction_block =
                                    if skip_next_instruction_pointer < instruction_blocks.len() {
                                        instruction_blocks[skip_next_instruction_pointer]
                                    } else {
                                        return Err(JitError::BranchTargetOutOfBounds {
                                            instruction_pointer: current_instruction_pointer,
                                            branch_target_instruction_pointer:
                                                skip_next_instruction_pointer,
                                            total_instruction_count: instruction_blocks.len(),
                                        });
                                    };
                                let proceed_to_next_instruction_block =
                                    if proceed_to_next_instruction_pointer
                                        < instruction_blocks.len()
                                    {
                                        instruction_blocks[proceed_to_next_instruction_pointer]
                                    } else {
                                        return Err(JitError::BranchTargetOutOfBounds {
                                            instruction_pointer: current_instruction_pointer,
                                            branch_target_instruction_pointer:
                                                proceed_to_next_instruction_pointer,
                                            total_instruction_count: instruction_blocks.len(),
                                        });
                                    };
                                function_builder.ins().brif(
                                    comparison_result,
                                    skip_next_instruction_block,
                                    &[],
                                    proceed_to_next_instruction_block,
                                    &[],
                                );
                                if skip_next_instruction_pointer < instruction_blocks.len()
                                    && !processed_instructions[skip_next_instruction_pointer]
                                {
                                    instruction_worklist.push_back(skip_next_instruction_pointer);
                                }
                                if proceed_to_next_instruction_pointer < instruction_blocks.len()
                                    && !processed_instructions[proceed_to_next_instruction_pointer]
                                {
                                    instruction_worklist
                                        .push_back(proceed_to_next_instruction_pointer);
                                }
                            }
                            _ => {
                                return Err(JitError::UnsupportedMemoryKind {
                                    instruction_pointer: current_instruction_pointer,
                                    operation: "LESS".to_string(),
                                    memory_kind_description: format!(
                                        "left: {:?}, right: {:?}",
                                        left.memory, right.memory
                                    ),
                                });
                            }
                        },
                        _ => {
                            return Err(JitError::UnsupportedOperandType {
                                instruction_pointer: current_instruction_pointer,
                                operand_type: r#type,
                                operation: "LESS".to_string(),
                            });
                        }
                    }
                }
                Operation::JUMP => {
                    let Jump {
                        offset,
                        is_positive,
                    } = Jump::from(*current_instruction);
                    let jump_target_instruction_pointer = if is_positive != 0 {
                        current_instruction_pointer + offset + 1
                    } else {
                        current_instruction_pointer - offset
                    };
                    if jump_target_instruction_pointer < instruction_blocks.len() {
                        if jump_target_instruction_pointer == current_instruction_pointer {
                            return Err(JitError::JumpToSelf {
                                instruction_pointer: current_instruction_pointer,
                            });
                        }
                        function_builder
                            .ins()
                            .jump(instruction_blocks[jump_target_instruction_pointer], &[]);
                        if !processed_instructions[jump_target_instruction_pointer] {
                            instruction_worklist.push_back(jump_target_instruction_pointer);
                        }
                    } else {
                        return Err(JitError::JumpTargetOutOfBounds {
                            instruction_pointer: current_instruction_pointer,
                            target_instruction_pointer: jump_target_instruction_pointer,
                            total_instruction_count: instruction_blocks.len(),
                        });
                    }
                }
                Operation::RETURN => {
                    let Return {
                        should_return_value,
                        return_value_address,
                        r#type,
                    } = Return::from(*current_instruction);

                    if should_return_value != 0 {
                        match r#type {
                            OperandType::INTEGER => {
                                let thread_runner_pointer =
                                    function_builder.use_var(Variable::new(0));
                                let call_frame_pointer = function_builder.use_var(Variable::new(1));
                                let call_frame_registers_field_offset =
                                    offset_of!(CallFrame, registers) as i32;
                                let call_frame_registers_pointer = function_builder.ins().load(
                                    pointer_type,
                                    MemFlags::new(),
                                    call_frame_pointer,
                                    call_frame_registers_field_offset,
                                );
                                let return_value_operand_index = function_builder
                                    .ins()
                                    .iconst(types::I64, return_value_address.index as i64);
                                let return_value_register_byte_offset =
                                    function_builder.ins().imul_imm(
                                        return_value_operand_index,
                                        std::mem::size_of::<Register>() as i64,
                                    );
                                let return_value_register_address = function_builder.ins().iadd(
                                    call_frame_registers_pointer,
                                    return_value_register_byte_offset,
                                );
                                let return_value = function_builder.ins().load(
                                    types::I64,
                                    MemFlags::new(),
                                    return_value_register_address,
                                    0,
                                );
                                function_builder.ins().call(
                                    set_return_function_reference,
                                    &[thread_runner_pointer, return_value],
                                );
                                function_builder.ins().return_(&[]);
                            }
                            _ => {
                                return Err(JitError::UnsupportedOperandType {
                                    instruction_pointer: current_instruction_pointer,
                                    operand_type: r#type,
                                    operation: "RETURN".to_string(),
                                });
                            }
                        }
                    } else {
                        function_builder.ins().return_(&[]);
                    }
                }
                _ => {
                    return Err(JitError::UnhandledOperation {
                        instruction_pointer: current_instruction_pointer,
                        operation_name: format!("{:?}", current_instruction.operation()),
                    });
                }
            }
            if !matches!(
                current_instruction.operation(),
                Operation::JUMP | Operation::RETURN | Operation::LESS
            ) {
                let next_sequential_instruction_pointer = current_instruction_pointer + 1;
                if next_sequential_instruction_pointer < instruction_blocks.len()
                    && bytecode_instructions[next_sequential_instruction_pointer].operation()
                        != Operation::RETURN
                {
                    function_builder
                        .ins()
                        .jump(instruction_blocks[next_sequential_instruction_pointer], &[]);
                    if !processed_instructions[next_sequential_instruction_pointer] {
                        instruction_worklist.push_back(next_sequential_instruction_pointer);
                    }
                }
            }
            processed_instructions[current_instruction_pointer] = true;
        }

        function_builder.switch_to_block(unreachable_final_block);
        function_builder.ins().return_(&[]);
        function_builder.seal_all_blocks();

        let compiled_function_id = self
            .module
            .declare_anonymous_function(&compilation_context.func.signature)
            .map_err(|error| JitError::CraneliftModuleError {
                instruction_pointer: None,
                message: format!("Failed to declare anonymous function: {error}"),
            })?;
        self.module
            .define_function(compiled_function_id, &mut compilation_context)
            .map_err(|error| JitError::FunctionCompilationError {
                message: format!("Failed to define function: {error}"),
            })?;
        self.module.clear_context(&mut compilation_context);
        self.module
            .finalize_definitions()
            .map_err(|error| JitError::CraneliftModuleError {
                instruction_pointer: None,
                message: format!("Failed to finalize definitions: {error}"),
            })?;

        let compiled_function_pointer = self.module.get_finalized_function(compiled_function_id);
        Ok(unsafe {
            std::mem::transmute::<*const u8, extern "C" fn(*mut ThreadRunner, *mut CallFrame)>(
                compiled_function_pointer,
            )
        })
    }
}

impl Default for Jit {
    fn default() -> Self {
        Self::new()
    }
}

pub struct JitInstruction {
    pub logic: extern "C" fn(*mut ThreadRunner, *mut CallFrame),
}

impl JitInstruction {
    pub fn no_op() -> Self {
        extern "C" fn no_op_logic(_: *mut ThreadRunner, _: *mut CallFrame) {}

        Self { logic: no_op_logic }
    }
}
