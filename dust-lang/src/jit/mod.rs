use std::mem::offset_of;

mod functions;
mod jit_error;

use functions::*;
pub use jit_error::JitError;

use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

use crate::{
    Address, CallFrame, Chunk, OperandType, Operation, Register, ThreadRunner,
    instruction::{Jump, Load, MemoryKind, Return, Test},
    vm::ObjectPool,
};

pub struct Jit<'a> {
    module: JITModule,
    chunk: &'a Chunk,
    object_pool: &'a mut ObjectPool,
}

impl<'a> Jit<'a> {
    pub fn new(chunk: &'a Chunk, object_pool: &'a mut ObjectPool) -> Self {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();

        builder.symbol(
            "set_return_value_to_integer",
            set_return_value_to_integer as *const u8,
        );

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
                instruction_pointer: ip,
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
        function_builder: &mut FunctionBuilder,
        call_frame_registers_pointer: Value,
        address: &Address,
    ) -> Result<Value, JitError> {
        match address.memory {
            MemoryKind::REGISTER => {
                let register_byte_offset = (address.index * size_of::<Register>()) as i32;
                Ok(function_builder.ins().load(
                    types::I64,
                    MemFlags::new(),
                    call_frame_registers_pointer,
                    register_byte_offset,
                ))
            }
            MemoryKind::CONSTANT => match self.chunk.constants[address.index].as_integer() {
                Some(val) => Ok(function_builder.ins().iconst(types::I64, val)),
                None => Err(JitError::InvalidConstantType {
                    constant_index: address.index,
                    expected_type: OperandType::INTEGER,
                }),
            },
            _ => Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            }),
        }
    }

    pub fn compile(&mut self) -> Result<JitInstruction, JitError> {
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

        let bytecode_instructions = &self.chunk.instructions;
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
        let set_return_value_to_integer_function_id = self
            .module
            .declare_function(
                "set_return_value_to_integer",
                Linkage::Import,
                &return_value_signature,
            )
            .map_err(|error| JitError::CraneliftModuleError {
                instruction_pointer: None,
                message: format!("Failed to declare set_return_value_integer function: {error}"),
            })?;
        let set_return_value_to_integer_function_reference = self.module.declare_func_in_func(
            set_return_value_to_integer_function_id,
            function_builder.func,
        );

        let function_entry_block = function_builder.create_block();
        function_builder.switch_to_block(function_entry_block);
        function_builder.append_block_params_for_function_params(function_entry_block);

        let variable_0 = function_builder.declare_var(pointer_type);
        let variable_1 = function_builder.declare_var(pointer_type);
        let variable_2 = function_builder.declare_var(pointer_type);
        let thread_runner_pointer = function_builder.block_params(function_entry_block)[0];
        let call_frame_pointer = function_builder.block_params(function_entry_block)[1];
        function_builder.def_var(variable_0, thread_runner_pointer);
        function_builder.def_var(variable_1, call_frame_pointer);

        let call_frame_registers_field_offset = offset_of!(CallFrame, registers) as i32;
        let call_frame_registers_pointer = function_builder.ins().load(
            pointer_type,
            MemFlags::new(),
            call_frame_pointer,
            call_frame_registers_field_offset,
        );
        function_builder.def_var(variable_2, call_frame_registers_pointer);

        function_builder.ins().jump(instruction_blocks[0], &[]);

        for ip in 0..total_instruction_count {
            function_builder.switch_to_block(instruction_blocks[ip]);
            let _thread_runner_pointer = function_builder.use_var(variable_0);
            let _call_frame_pointer = function_builder.use_var(variable_1);
            let registers_pointer = function_builder.use_var(variable_2);
            let current_instruction = &bytecode_instructions[ip];
            let operation = current_instruction.operation();

            match operation {
                Operation::LOAD => {
                    let Load {
                        destination,
                        operand,
                        r#type,
                        ..
                    } = Load::from(*current_instruction);
                    let value = match r#type {
                        OperandType::INTEGER => {
                            self.get_integer(&mut function_builder, registers_pointer, &operand)?
                        }
                        // OperandType::FLOAT => self.get_float(...)?,
                        // OperandType::STRING => self.get_string(...)?,
                        _ => {
                            return Err(JitError::UnsupportedOperandType {
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
                    match r#type {
                        OperandType::INTEGER => {
                            let left_integer =
                                self.get_integer(&mut function_builder, registers_pointer, &left)?;
                            let right_integer =
                                self.get_integer(&mut function_builder, registers_pointer, &right)?;
                            let result_value = match current_instruction.operation() {
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
                                _ => unreachable!(),
                            };
                            let destination_register_byte_offset =
                                (destination.index * size_of::<Register>()) as i32;

                            function_builder.ins().store(
                                MemFlags::new(),
                                result_value,
                                registers_pointer,
                                destination_register_byte_offset,
                            );
                        }
                        _ => {
                            return Err(JitError::UnsupportedOperandType {
                                operand_type: r#type,
                            });
                        }
                    }

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
                            self.get_integer(&mut function_builder, registers_pointer, &left)?,
                            self.get_integer(&mut function_builder, registers_pointer, &right)?,
                        ),
                        _ => {
                            return Err(JitError::UnsupportedOperandType {
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
                                instruction_pointer: ip,
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
                            instruction_pointer: ip,
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
                            instruction_pointer: ip,
                            target_instruction_pointer: jump_target_ip,
                            total_instruction_count: instruction_blocks.len(),
                        });
                    }

                    if jump_target_ip == ip {
                        return Err(JitError::JumpToSelf {
                            instruction_pointer: ip,
                        });
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
                            OperandType::INTEGER => {
                                let return_value_register_byte_offset =
                                    (return_value_address.index * size_of::<Register>()) as i32;
                                let return_value = function_builder.ins().load(
                                    types::I64,
                                    MemFlags::new(),
                                    registers_pointer,
                                    return_value_register_byte_offset,
                                );
                                function_builder.ins().call(
                                    set_return_value_to_integer_function_reference,
                                    &[thread_runner_pointer, return_value],
                                );
                                function_builder.ins().return_(&[]);
                            }
                            _ => {
                                return Err(JitError::UnsupportedOperandType {
                                    operand_type: r#type,
                                });
                            }
                        }
                    } else {
                        function_builder.ins().return_(&[]);
                    }
                }
                _ => {
                    return Err(JitError::UnhandledOperation {
                        instruction_pointer: ip,
                        operation_name: format!("{:?}", current_instruction.operation()),
                    });
                }
            }
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
        let logic = unsafe {
            std::mem::transmute::<*const u8, extern "C" fn(*mut ThreadRunner, *mut CallFrame)>(
                compiled_function_pointer,
            )
        };

        Ok(JitInstruction {
            logic,
            register_count: self.chunk.register_tags.len(),
        })
    }
}

pub struct JitInstruction {
    pub logic: extern "C" fn(*mut ThreadRunner, *mut CallFrame),
    pub register_count: usize,
}

impl JitInstruction {
    pub fn no_op() -> Self {
        extern "C" fn no_op_logic(_: *mut ThreadRunner, _: *mut CallFrame) {}

        Self {
            logic: no_op_logic,
            register_count: 0,
        }
    }
}
