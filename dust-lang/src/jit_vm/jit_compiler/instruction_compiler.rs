use std::{collections::HashSet, mem::offset_of};

use cranelift::{
    codegen::ir::FuncRef,
    prelude::{
        AbiParam, Block, FunctionBuilder, InstBuilder, IntCC, MemFlags, Signature,
        Value as CraneliftValue, Variable,
        types::{F64, I8, I64},
    },
};
use cranelift_jit::JITModule;
use cranelift_module::{FuncId, Linkage, Module};
use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;
use tracing::trace;

use crate::{
    constant_table::ConstantTable,
    instruction::{
        Add, Address, Call, CallNative, Divide, Drop, GetList, Instruction, Jump, MemoryKind,
        Modulo, Move, Multiply, Negate, NewList, OperandType, Operation, Power, Return, SetList,
        Subtract, Test, ToString,
    },
    jit_vm::{
        JitError, RegisterTag,
        thread::{JitPrototype, ThreadContextFields},
    },
    native_function::NativeFunction,
    prototype::Prototype,
};

pub struct InstructionCompiler<'a> {
    pub prototype: &'a Prototype,
    pub instruction_blocks: &'a [Block],
    pub function_ids: &'a [FuncId],

    pub constants: &'a ConstantTable,
    pub ssa_registers: &'a mut [Variable],

    pub thread_context: CraneliftValue,
    pub thread_context_fields: ThreadContextFields,
    pub base_register_index: CraneliftValue,
    pub recursive_calls: &'a HashSet<(u16, u16), FxBuildHasher>,

    pub module: &'a mut JITModule,
    pub signature: Signature,
}

impl<'a> InstructionCompiler<'a> {
    pub fn compile(&mut self, ip: usize, builder: &mut FunctionBuilder) -> Result<(), JitError> {
        builder.switch_to_block(self.instruction_blocks[ip]);

        let instruction =
            self.prototype
                .instructions
                .get(ip)
                .ok_or(JitError::InstructionIndexOutOfBounds {
                    instruction_index: ip,
                    total_instruction_count: self.prototype.instructions.len(),
                })?;
        let operation = instruction.operation();

        #[cfg(debug_assertions)]
        {
            trace!(
                "JIT compiling {operation} at IP {ip} for proto_{}",
                self.prototype.index
            );

            let log_function = self.get_log_operation_and_ip_function(builder)?;
            let operation_value = builder.ins().iconst(I8, operation.0 as i64);
            let ip_value = builder.ins().iconst(I64, ip as i64);

            builder
                .ins()
                .call(log_function, &[operation_value, ip_value]);
        }

        match operation {
            Operation::MOVE => self.compile_move(instruction, ip, builder),
            Operation::TEST => self.compile_test(instruction, ip, builder),
            Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL => {
                self.compile_comparison(instruction, ip, operation, builder)
            }
            Operation::CALL => self.compile_call(instruction, ip, builder),
            Operation::CALL_NATIVE => self.compile_call_native(instruction, ip, builder),
            Operation::NEGATE => self.compile_negate(instruction, ip, builder),
            Operation::ADD => self.compile_add(instruction, ip, builder),
            Operation::SUBTRACT => self.compile_subtract(instruction, ip, builder),
            Operation::MULTIPLY => self.compile_multiply(instruction, ip, builder),
            Operation::DIVIDE => self.compile_divide(instruction, ip, builder),
            Operation::MODULO => self.compile_modulo(instruction, ip, builder),
            Operation::POWER => self.compile_power(instruction, ip, builder),
            Operation::NEW_LIST => self.compile_new_list(instruction, ip, builder),
            Operation::GET_LIST => self.compile_get_list(instruction, ip, builder),
            Operation::SET_LIST => self.compile_set_list(instruction, ip, builder),
            Operation::TO_STRING => self.compile_to_string(instruction, ip, builder),
            Operation::JUMP => self.compile_jump(instruction, ip, builder),
            Operation::DROP => self.compile_drop(instruction, ip, builder),
            Operation::RETURN => self.compile_return(instruction, builder),
            _ => Err(JitError::UnsupportedOperation { operation }),
        }
    }

    fn compile_move(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let Move {
            destination,
            operand,
            r#type,
            jump_distance,
            jump_is_positive,
        } = Move::from(instruction);

        let operand_value = self.get_value(operand, r#type, builder)?;

        self.set_register_and_tag(destination, operand_value, r#type, builder)?;

        if jump_distance > 0 {
            let distance = (jump_distance + 1) as usize;

            if jump_is_positive {
                builder
                    .ins()
                    .jump(self.instruction_blocks[ip + distance], &[]);
            } else {
                builder
                    .ins()
                    .jump(self.instruction_blocks[ip - distance], &[]);
            }
        } else {
            builder.ins().jump(self.instruction_blocks[ip + 1], &[]);
        }

        Ok(())
    }

    fn compile_test(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let Test {
            comparator,
            operand,
            jump_distance,
        } = Test::from(instruction);

        let operand_value = self.get_boolean(operand, builder)?;
        let false_value = builder.ins().iconst(I8, 0);
        let condition = if comparator {
            IntCC::NotEqual
        } else {
            IntCC::Equal
        };
        let comparison_result = builder.ins().icmp(condition, operand_value, false_value);
        let target_ip = ip + (jump_distance as usize) + 1;

        builder.ins().brif(
            comparison_result,
            self.instruction_blocks[target_ip],
            &[],
            self.instruction_blocks[ip + 1],
            &[],
        );

        Ok(())
    }

    fn compile_comparison(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        operation: Operation,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let comparator = instruction.a_field() != 0;
        let left = instruction.b_address();
        let right = instruction.c_address();
        let r#type = instruction.operand_type();

        let left_value = self.get_value(left, r#type, builder)?;
        let right_value = self.get_value(right, r#type, builder)?;
        let comparison_result = match r#type {
            OperandType::STRING => {
                let compare_strings_function = match operation {
                    Operation::EQUAL => self.get_compare_strings_equal_function(builder)?,
                    Operation::LESS => self.get_compare_strings_less_than_function(builder)?,
                    Operation::LESS_EQUAL => {
                        self.get_compare_strings_less_than_equal_function(builder)?
                    }
                    _ => {
                        return Err(JitError::UnsupportedOperation { operation });
                    }
                };
                let call_compare_strings = builder
                    .ins()
                    .call(compare_strings_function, &[left_value, right_value]);
                let compare_result = builder.inst_results(call_compare_strings)[0];

                if comparator {
                    compare_result
                } else {
                    builder.ins().bxor_imm(compare_result, 1)
                }
            }
            OperandType::LIST_BOOLEAN
            | OperandType::LIST_BYTE
            | OperandType::LIST_CHARACTER
            | OperandType::LIST_FLOAT
            | OperandType::LIST_INTEGER
            | OperandType::LIST_STRING
            | OperandType::LIST_FUNCTION
            | OperandType::LIST_LIST => {
                let compare_lists_function = match operation {
                    Operation::EQUAL => self.get_compare_lists_equal_function(builder)?,
                    Operation::LESS => self.get_compare_lists_less_than_function(builder)?,
                    Operation::LESS_EQUAL => {
                        self.get_compare_lists_less_than_equal_function(builder)?
                    }
                    _ => {
                        return Err(JitError::UnsupportedOperation { operation });
                    }
                };
                let call_compare_lists = builder
                    .ins()
                    .call(compare_lists_function, &[left_value, right_value]);
                let compare_result = builder.inst_results(call_compare_lists)[0];

                if comparator {
                    compare_result
                } else {
                    builder.ins().bxor_imm(compare_result, 1)
                }
            }
            _ => {
                let condition = match (operation, comparator) {
                    (Operation::EQUAL, true) => IntCC::Equal,
                    (Operation::EQUAL, false) => IntCC::NotEqual,
                    (Operation::LESS, true) => IntCC::SignedLessThan,
                    (Operation::LESS, false) => IntCC::SignedGreaterThanOrEqual,
                    (Operation::LESS_EQUAL, true) => IntCC::SignedLessThanOrEqual,
                    (Operation::LESS_EQUAL, false) => IntCC::SignedGreaterThan,
                    _ => {
                        return Err(JitError::UnsupportedOperation { operation });
                    }
                };

                builder.ins().icmp(condition, left_value, right_value)
            }
        };

        builder.ins().brif(
            comparison_result,
            self.instruction_blocks[ip + 2],
            &[],
            self.instruction_blocks[ip + 1],
            &[],
        );

        Ok(())
    }

    fn compile_call(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let Call {
            destination,
            callee,
            arguments_start,
            argument_count,
        } = Call::from(instruction);

        // let caller_prototype_index = builder.ins().iconst(I64, self.prototype.index as i64);
        let callee_prototype_index = self.get_prototype_index(callee, builder)?;
        let jit_prototype_offset = builder
            .ins()
            .imul_imm(callee_prototype_index, size_of::<JitPrototype>() as i64);
        let compiled_prototype_address = builder.ins().iadd(
            self.thread_context_fields.jit_prototype_buffer_pointer,
            jit_prototype_offset,
        );
        let callee_pointer = builder.ins().load(
            I64,
            MemFlags::new(),
            compiled_prototype_address,
            offset_of!(JitPrototype, function_pointer) as i32,
        );
        let return_value_tag = builder.ins().load(
            I8,
            MemFlags::new(),
            compiled_prototype_address,
            offset_of!(JitPrototype, return_value_tag) as i32,
        );

        // let recursive_block = builder.create_block();
        // let non_recursive_block = builder.create_block();

        // let is_recursive =
        //     builder
        //         .ins()
        //         .icmp(IntCC::Equal, caller_prototype_index, callee_prototype_index);

        // builder
        //     .ins()
        //     .brif(is_recursive, recursive_block, &[], non_recursive_block, &[]);

        // builder.switch_to_block(non_recursive_block);

        let arguments_end = (arguments_start + argument_count) as usize;
        let argument_range = arguments_start as usize..arguments_end;

        for argument_index in argument_range {
            let (address, r#type) = self.prototype.call_arguments.get(argument_index).ok_or(
                JitError::CallArgumentIndexOutOfBounds {
                    argument_index,
                    total_argument_count: self.prototype.call_arguments.len(),
                },
            )?;
            let argument_value = self.get_value(*address, *r#type, builder)?;

            let argument_index_value = builder.ins().iconst(I64, argument_index as i64);
            let argument_register_offset = builder
                .ins()
                .imul_imm(argument_index_value, size_of::<CraneliftValue>() as i64);
            let argument_register_address = builder.ins().iadd(
                self.thread_context_fields.function_arguments,
                argument_register_offset,
            );

            builder.ins().store(
                MemFlags::new(),
                argument_value,
                argument_register_address,
                0,
            );
        }

        let signature_reference = builder.import_signature(self.signature.clone());
        let call_callee = builder.ins().call_indirect(
            signature_reference,
            callee_pointer,
            &[self.thread_context, self.base_register_index],
        );

        let empty_tag = builder.ins().iconst(I8, RegisterTag::EMPTY.0 as i64);
        let function_returns_value =
            builder
                .ins()
                .icmp(IntCC::NotEqual, return_value_tag, empty_tag);
        let set_return_value_block = builder.create_block();

        builder.ins().brif(
            function_returns_value,
            set_return_value_block,
            &[],
            self.instruction_blocks[ip + 1],
            &[],
        );
        builder.switch_to_block(set_return_value_block);

        if destination != u16::MAX {
            let return_value = builder.inst_results(call_callee)[0];

            self.ssa_registers
                .get(destination as usize)
                .map(|ssa_variable| builder.def_var(*ssa_variable, return_value))
                .ok_or(JitError::RegisterIndexOutOfBounds {
                    register_index: destination,
                    total_register_count: self.ssa_registers.len(),
                })?;
            self.set_register_tag(destination, return_value_tag, builder)?;
        }

        builder.ins().jump(self.instruction_blocks[ip + 1], &[]);

        Ok(())
    }

    fn compile_call_native(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let CallNative {
            destination,
            function,
            arguments_start,
            argument_count,
            return_type,
        } = CallNative::from(instruction);

        let arguments_end = (arguments_start + argument_count) as usize;
        let argument_range = arguments_start as usize..arguments_end;
        let mut argument_values =
            SmallVec::<[CraneliftValue; 8]>::with_capacity(argument_count as usize + 1);

        for argument_index in argument_range {
            let (address, r#type) = self.prototype.call_arguments.get(argument_index).ok_or(
                JitError::CallArgumentIndexOutOfBounds {
                    argument_index,
                    total_argument_count: self.prototype.call_arguments.len(),
                },
            )?;
            let argument_value = self.get_value(*address, *r#type, builder)?;

            argument_values.push(argument_value);
        }

        let callee_reference = match function {
            NativeFunction::READ_LINE => {
                let function = self.get_read_line_function(builder)?;

                argument_values.push(self.thread_context);

                function
            }
            NativeFunction::WRITE_LINE => self.get_write_line_function(builder)?,
            _ => {
                return Err(JitError::UnsupportedNativeFunction {
                    function_name: function.name(),
                });
            }
        };
        let call_callee = builder.ins().call(callee_reference, &argument_values);

        if return_type != OperandType::NONE {
            let return_value = builder.inst_results(call_callee)[0];

            self.set_register_and_tag(destination, return_value, return_type, builder)?;
        }

        builder.ins().jump(self.instruction_blocks[ip + 1], &[]);

        Ok(())
    }

    fn compile_negate(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let Negate {
            destination,
            operand,
            r#type,
        } = Negate::from(instruction);

        let negated_value = match r#type {
            OperandType::BOOLEAN => {
                let boolean_value = self.get_boolean(operand, builder)?;
                let one = builder.ins().iconst(I8, 1);
                let negated_boolean = builder.ins().bxor(boolean_value, one);

                builder.ins().uextend(I64, negated_boolean)
            }
            OperandType::INTEGER => {
                let integer_value = self.get_integer(operand, builder)?;

                builder.ins().ineg(integer_value)
            }
            OperandType::FLOAT => {
                let float_value = self.get_float(operand, builder)?;
                let negated_float = builder.ins().fneg(float_value);

                builder.ins().bitcast(I64, MemFlags::new(), negated_float)
            }
            _ => {
                return Err(JitError::UnsupportedOperandType {
                    operand_type: r#type,
                });
            }
        };

        self.set_register_and_tag(destination, negated_value, r#type, builder)?;
        builder.ins().jump(self.instruction_blocks[ip + 1], &[]);

        Ok(())
    }

    fn compile_add(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let Add {
            destination,
            left,
            right,
            r#type,
        } = Add::from(instruction);

        let sum_value = match r#type {
            OperandType::BYTE => {
                let left_byte = self.get_byte(left, builder)?;
                let right_byte = self.get_byte(right, builder)?;
                let sum_byte = builder.ins().iadd(left_byte, right_byte);

                builder.ins().uextend(I64, sum_byte)
            }
            OperandType::INTEGER => {
                let left_integer = self.get_integer(left, builder)?;
                let right_integer = self.get_integer(right, builder)?;

                builder.ins().iadd(left_integer, right_integer)
            }
            OperandType::FLOAT => {
                let left_float = self.get_float(left, builder)?;
                let right_float = self.get_float(right, builder)?;
                let sum_float = builder.ins().fadd(left_float, right_float);

                builder.ins().bitcast(I64, MemFlags::new(), sum_float)
            }
            OperandType::STRING => {
                let left_string = self.get_string(left, builder)?;
                let right_string = self.get_string(right, builder)?;
                let concatenate_strings_function =
                    self.get_concatenate_strings_function(builder)?;
                let call_concatenate_strings = builder.ins().call(
                    concatenate_strings_function,
                    &[left_string, right_string, self.thread_context],
                );

                builder.inst_results(call_concatenate_strings)[0]
            }
            OperandType::CHARACTER => {
                let left_character = self.get_character(left, builder)?;
                let right_character = self.get_character(right, builder)?;
                let concatenate_characters_function =
                    self.get_concatenate_characters_function(builder)?;
                let call_concatenate_characters = builder.ins().call(
                    concatenate_characters_function,
                    &[left_character, right_character, self.thread_context],
                );

                builder.inst_results(call_concatenate_characters)[0]
            }
            OperandType::STRING_CHARACTER => {
                let left_string = self.get_string(left, builder)?;
                let right_character = self.get_character(right, builder)?;
                let concatenate_string_character_function =
                    self.get_concatenate_string_character_function(builder)?;
                let call_concatenate_string_character = builder.ins().call(
                    concatenate_string_character_function,
                    &[left_string, right_character, self.thread_context],
                );

                builder.inst_results(call_concatenate_string_character)[0]
            }
            OperandType::CHARACTER_STRING => {
                let left_character = self.get_character(left, builder)?;
                let right_string = self.get_string(right, builder)?;
                let concatenate_character_string_function =
                    self.get_concatenate_character_string_function(builder)?;
                let call_concatenate_character_string = builder.ins().call(
                    concatenate_character_string_function,
                    &[left_character, right_string, self.thread_context],
                );

                builder.inst_results(call_concatenate_character_string)[0]
            }
            _ => {
                return Err(JitError::UnsupportedOperandType {
                    operand_type: r#type,
                });
            }
        };
        let result_type = r#type.destination_type();

        self.set_register_and_tag(destination, sum_value, result_type, builder)?;
        builder.ins().jump(self.instruction_blocks[ip + 1], &[]);

        Ok(())
    }

    fn compile_subtract(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let Subtract {
            destination,
            left,
            right,
            r#type,
        } = Subtract::from(instruction);

        let difference_value = match r#type {
            OperandType::BYTE => {
                let left_byte = self.get_byte(left, builder)?;
                let right_byte = self.get_byte(right, builder)?;
                let difference_byte = builder.ins().isub(left_byte, right_byte);

                builder.ins().uextend(I64, difference_byte)
            }
            OperandType::INTEGER => {
                let left_integer = self.get_integer(left, builder)?;
                let right_integer = self.get_integer(right, builder)?;

                builder.ins().isub(left_integer, right_integer)
            }
            OperandType::FLOAT => {
                let left_float = self.get_float(left, builder)?;
                let right_float = self.get_float(right, builder)?;
                let difference_float = builder.ins().fsub(left_float, right_float);

                builder
                    .ins()
                    .bitcast(I64, MemFlags::new(), difference_float)
            }
            _ => {
                return Err(JitError::UnsupportedOperandType {
                    operand_type: r#type,
                });
            }
        };

        self.set_register_and_tag(destination, difference_value, r#type, builder)?;
        builder.ins().jump(self.instruction_blocks[ip + 1], &[]);

        Ok(())
    }

    fn compile_multiply(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let Multiply {
            destination,
            left,
            right,
            r#type,
        } = Multiply::from(instruction);

        let product_value = match r#type {
            OperandType::BYTE => {
                let left_byte = self.get_byte(left, builder)?;
                let right_byte = self.get_byte(right, builder)?;
                let product_byte = builder.ins().imul(left_byte, right_byte);

                builder.ins().uextend(I64, product_byte)
            }
            OperandType::INTEGER => {
                let left_integer = self.get_integer(left, builder)?;
                let right_integer = self.get_integer(right, builder)?;

                builder.ins().imul(left_integer, right_integer)
            }
            OperandType::FLOAT => {
                let left_float = self.get_float(left, builder)?;
                let right_float = self.get_float(right, builder)?;
                let product_float = builder.ins().fmul(left_float, right_float);

                builder.ins().bitcast(I64, MemFlags::new(), product_float)
            }
            _ => {
                return Err(JitError::UnsupportedOperandType {
                    operand_type: r#type,
                });
            }
        };

        self.set_register_and_tag(destination, product_value, r#type, builder)?;
        builder.ins().jump(self.instruction_blocks[ip + 1], &[]);

        Ok(())
    }

    fn compile_divide(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let Divide {
            destination,
            left,
            right,
            r#type,
        } = Divide::from(instruction);

        let quotient_value = match r#type {
            OperandType::BYTE => {
                let left_byte = self.get_byte(left, builder)?;
                let right_byte = self.get_byte(right, builder)?;
                let quotient_byte = builder.ins().udiv(left_byte, right_byte);

                builder.ins().uextend(I64, quotient_byte)
            }
            OperandType::INTEGER => {
                let left_integer = self.get_integer(left, builder)?;
                let right_integer = self.get_integer(right, builder)?;

                builder.ins().sdiv(left_integer, right_integer)
            }
            OperandType::FLOAT => {
                let left_float = self.get_float(left, builder)?;
                let right_float = self.get_float(right, builder)?;
                let quotient_float = builder.ins().fdiv(left_float, right_float);

                builder.ins().bitcast(I64, MemFlags::new(), quotient_float)
            }
            _ => {
                return Err(JitError::UnsupportedOperandType {
                    operand_type: r#type,
                });
            }
        };

        self.set_register_and_tag(destination, quotient_value, r#type, builder)?;
        builder.ins().jump(self.instruction_blocks[ip + 1], &[]);

        Ok(())
    }

    fn compile_modulo(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let Modulo {
            destination,
            left,
            right,
            r#type,
        } = Modulo::from(instruction);

        let remainder_value = match r#type {
            OperandType::BYTE => {
                let left_byte = self.get_byte(left, builder)?;
                let right_byte = self.get_byte(right, builder)?;
                let remainder_byte = builder.ins().urem(left_byte, right_byte);

                builder.ins().uextend(I64, remainder_byte)
            }
            OperandType::FLOAT => {
                let left_float = self.get_float(left, builder)?;
                let right_float = self.get_float(right, builder)?;
                let quotient_float = builder.ins().fdiv(left_float, right_float);
                let floored_quotient = builder.ins().floor(quotient_float);
                let multiplied = builder.ins().fmul(floored_quotient, right_float);
                let remainder_float = builder.ins().fsub(left_float, multiplied);

                builder.ins().bitcast(I64, MemFlags::new(), remainder_float)
            }
            OperandType::INTEGER => {
                let left_integer = self.get_integer(left, builder)?;
                let right_integer = self.get_integer(right, builder)?;

                builder.ins().srem(left_integer, right_integer)
            }
            _ => {
                return Err(JitError::UnsupportedOperandType {
                    operand_type: r#type,
                });
            }
        };

        self.set_register_and_tag(destination, remainder_value, r#type, builder)?;
        builder.ins().jump(self.instruction_blocks[ip + 1], &[]);

        Ok(())
    }

    fn compile_power(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let Power {
            destination,
            base,
            exponent,
            r#type,
        } = Power::from(instruction);

        let power_value = match r#type {
            OperandType::BYTE => {
                let base_byte = self.get_byte(base, builder)?;
                let exponent_byte = self.get_byte(exponent, builder)?;
                let byte_power_function = self.get_byte_power_function(builder)?;
                let call_byte_power = builder
                    .ins()
                    .call(byte_power_function, &[base_byte, exponent_byte]);

                builder.inst_results(call_byte_power)[0]
            }
            OperandType::FLOAT => {
                let base_float = self.get_float(base, builder)?;
                let exponent_float = self.get_float(exponent, builder)?;
                let float_power_function = self.get_float_power_function(builder)?;
                let call_float_power = builder
                    .ins()
                    .call(float_power_function, &[base_float, exponent_float]);

                builder.inst_results(call_float_power)[0]
            }
            OperandType::INTEGER => {
                let base_integer = self.get_integer(base, builder)?;
                let exponent_integer = self.get_integer(exponent, builder)?;
                let integer_power_function = self.get_integer_power_function(builder)?;
                let call_integer_power = builder
                    .ins()
                    .call(integer_power_function, &[base_integer, exponent_integer]);

                builder.inst_results(call_integer_power)[0]
            }
            _ => {
                return Err(JitError::UnsupportedOperandType {
                    operand_type: r#type,
                });
            }
        };

        self.set_register_and_tag(destination, power_value, r#type, builder)?;
        builder.ins().jump(self.instruction_blocks[ip + 1], &[]);

        Ok(())
    }

    fn compile_new_list(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let NewList {
            destination,
            initial_length,
            list_type,
        } = NewList::from(instruction);

        let initial_length_value = self.get_integer(initial_length, builder)?;
        let list_type_value = builder.ins().iconst(I8, list_type.0 as i64);
        let allocate_list_function = self.get_allocate_list_function(builder)?;
        let call_allocate_list = builder.ins().call(
            allocate_list_function,
            &[list_type_value, initial_length_value, self.thread_context],
        );
        let list_object_pointer = builder.inst_results(call_allocate_list)[0];

        self.set_register_and_tag(destination, list_object_pointer, list_type, builder)?;
        builder.ins().jump(self.instruction_blocks[ip + 1], &[]);

        Ok(())
    }

    fn compile_get_list(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let GetList {
            destination,
            list,
            list_index,
            item_type,
        } = GetList::from(instruction);

        let list_object_pointer = self
            .ssa_registers
            .get(list.index as usize)
            .map(|ssa_variable| builder.use_var(*ssa_variable))
            .ok_or(JitError::RegisterIndexOutOfBounds {
                register_index: list.index,
                total_register_count: self.ssa_registers.len(),
            })?;
        let index_value = self.get_integer(list_index, builder)?;
        let get_from_list_function = self.get_get_from_list_function(builder)?;
        let call_get_from_list = builder.ins().call(
            get_from_list_function,
            &[list_object_pointer, index_value, self.thread_context],
        );
        let item_value = builder.inst_results(call_get_from_list)[0];

        self.set_register_and_tag(destination, item_value, item_type, builder)?;
        builder.ins().jump(self.instruction_blocks[ip + 1], &[]);

        Ok(())
    }

    fn compile_set_list(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let SetList {
            destination_list,
            item_source,
            index,
            item_type,
        } = SetList::from(instruction);

        let list_object_pointer = self
            .ssa_registers
            .get(destination_list as usize)
            .map(|ssa_variable| builder.use_var(*ssa_variable))
            .ok_or(JitError::RegisterIndexOutOfBounds {
                register_index: destination_list,
                total_register_count: self.ssa_registers.len(),
            })?;
        let index_value = self.get_integer(index, builder)?;
        let item_value = self.get_value(item_source, item_type, builder)?;
        let insert_into_list_function = self.get_insert_into_list_function(builder)?;

        builder.ins().call(
            insert_into_list_function,
            &[list_object_pointer, index_value, item_value],
        );
        builder.ins().jump(self.instruction_blocks[ip + 1], &[]);

        Ok(())
    }

    fn compile_to_string(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let ToString {
            destination,
            operand,
            r#type,
        } = ToString::from(instruction);

        let operand_value = self.get_value(operand, r#type, builder)?;
        let to_string_function = match r#type {
            OperandType::INTEGER => self.get_integer_to_string_function(builder)?,
            _ => {
                return Err(JitError::UnsupportedOperandType {
                    operand_type: r#type,
                });
            }
        };
        let call_to_string = builder
            .ins()
            .call(to_string_function, &[operand_value, self.thread_context]);
        let string_value = builder.inst_results(call_to_string)[0];

        self.set_register_and_tag(destination, string_value, OperandType::STRING, builder)?;
        builder.ins().jump(self.instruction_blocks[ip + 1], &[]);

        Ok(())
    }

    fn compile_jump(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let Jump {
            offset,
            is_positive,
            drop_list_start,
            drop_list_end,
        } = Jump::from(instruction);
        let target_ip = if is_positive {
            ip + (offset as usize) + 1
        } else {
            ip - (offset as usize) - 1
        };

        if drop_list_end != 0 {
            for drop_index in drop_list_start..drop_list_end {
                let register_index = *self.prototype.drop_lists.get(drop_index as usize).ok_or(
                    JitError::DropListIndexOutOfBounds {
                        drop_list_index: drop_index,
                        drop_list_length: self.prototype.drop_lists.len(),
                    },
                )?;
                let tag_value = builder.ins().iconst(I8, RegisterTag::EMPTY.0 as i64);

                self.set_register_tag(register_index, tag_value, builder)?;
            }
        }

        builder.ins().jump(self.instruction_blocks[target_ip], &[]);

        Ok(())
    }

    fn compile_drop(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let Drop {
            drop_list_start,
            drop_list_end,
        } = Drop::from(instruction);

        for drop_index in drop_list_start..drop_list_end {
            let register_index = *self.prototype.drop_lists.get(drop_index as usize).ok_or(
                JitError::DropListIndexOutOfBounds {
                    drop_list_index: drop_index,
                    drop_list_length: self.prototype.drop_lists.len(),
                },
            )?;
            let tag_value = builder.ins().iconst(I8, RegisterTag::EMPTY.0 as i64);

            self.set_register_tag(register_index, tag_value, builder)?;
        }

        builder.ins().jump(self.instruction_blocks[ip + 1], &[]);

        Ok(())
    }

    fn compile_return(
        &mut self,
        instruction: &Instruction,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let Return { operand, r#type } = Return::from(instruction);

        let return_value = if r#type == OperandType::NONE {
            builder.ins().iconst(I64, 0)
        } else {
            self.get_value(operand, r#type, builder)?
        };

        builder.ins().return_(&[return_value]);

        Ok(())
    }

    fn get_value(
        &mut self,
        address: Address,
        r#type: OperandType,
        builder: &mut FunctionBuilder,
    ) -> Result<CraneliftValue, JitError> {
        match r#type {
            OperandType::BOOLEAN => match address.memory {
                MemoryKind::REGISTER => self
                    .ssa_registers
                    .get(address.index as usize)
                    .map(|ssa_variable| builder.use_var(*ssa_variable))
                    .ok_or(JitError::RegisterIndexOutOfBounds {
                        register_index: address.index,
                        total_register_count: self.ssa_registers.len(),
                    }),
                MemoryKind::ENCODED => {
                    let boolean = address.index != 0;
                    let value = builder.ins().iconst(I64, if boolean { 1 } else { 0 });

                    Ok(value)
                }
                _ => Err(JitError::UnsupportedMemoryKind {
                    memory_kind: address.memory,
                }),
            },
            OperandType::BYTE => match address.memory {
                MemoryKind::REGISTER => self
                    .ssa_registers
                    .get(address.index as usize)
                    .map(|ssa_variable| builder.use_var(*ssa_variable))
                    .ok_or(JitError::RegisterIndexOutOfBounds {
                        register_index: address.index,
                        total_register_count: self.ssa_registers.len(),
                    }),
                MemoryKind::ENCODED => {
                    let byte = address.index as u8;
                    let byte_value = builder.ins().iconst(I64, byte as i64);

                    Ok(byte_value)
                }
                _ => Err(JitError::UnsupportedMemoryKind {
                    memory_kind: address.memory,
                }),
            },
            OperandType::CHARACTER => self.get_character(address, builder),
            OperandType::FLOAT => match address.memory {
                MemoryKind::REGISTER => self
                    .ssa_registers
                    .get(address.index as usize)
                    .map(|ssa_variable| builder.use_var(*ssa_variable))
                    .ok_or(JitError::RegisterIndexOutOfBounds {
                        register_index: address.index,
                        total_register_count: self.ssa_registers.len(),
                    }),
                MemoryKind::CONSTANT => {
                    let float = self.constants.get_float(address.index).ok_or(
                        JitError::ConstantIndexOutOfBounds {
                            constant_index: address.index,
                            total_constant_count: self.constants.len(),
                        },
                    )?;
                    let value = builder.ins().iconst(I64, float.to_bits() as i64);

                    Ok(value)
                }
                _ => Err(JitError::UnsupportedMemoryKind {
                    memory_kind: address.memory,
                }),
            },
            OperandType::INTEGER => self.get_integer(address, builder),
            OperandType::STRING => self.get_string(address, builder),
            OperandType::LIST_BOOLEAN
            | OperandType::LIST_BYTE
            | OperandType::LIST_CHARACTER
            | OperandType::LIST_FLOAT
            | OperandType::LIST_INTEGER
            | OperandType::LIST_STRING
            | OperandType::LIST_FUNCTION
            | OperandType::LIST_LIST => self
                .ssa_registers
                .get(address.index as usize)
                .map(|ssa_variable| builder.use_var(*ssa_variable))
                .ok_or(JitError::RegisterIndexOutOfBounds {
                    register_index: address.index,
                    total_register_count: self.ssa_registers.len(),
                }),
            _ => Err(JitError::UnsupportedOperandType {
                operand_type: r#type,
            }),
        }
    }

    fn get_boolean(
        &mut self,
        address: Address,
        builder: &mut FunctionBuilder,
    ) -> Result<CraneliftValue, JitError> {
        match address.memory {
            MemoryKind::REGISTER => self
                .ssa_registers
                .get(address.index as usize)
                .map(|ssa_variable| {
                    let i64_value = builder.use_var(*ssa_variable);

                    builder.ins().ireduce(I8, i64_value)
                })
                .ok_or(JitError::RegisterIndexOutOfBounds {
                    register_index: address.index,
                    total_register_count: self.ssa_registers.len(),
                }),
            MemoryKind::ENCODED => {
                let boolean = address.index != 0;
                let value = builder.ins().iconst(I8, if boolean { 1 } else { 0 });

                Ok(value)
            }
            _ => Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            }),
        }
    }

    fn get_byte(
        &mut self,
        address: Address,
        builder: &mut FunctionBuilder,
    ) -> Result<CraneliftValue, JitError> {
        match address.memory {
            MemoryKind::REGISTER => self
                .ssa_registers
                .get(address.index as usize)
                .map(|ssa_variable| {
                    let i64_value = builder.use_var(*ssa_variable);

                    builder.ins().ireduce(I8, i64_value)
                })
                .ok_or(JitError::RegisterIndexOutOfBounds {
                    register_index: address.index,
                    total_register_count: self.ssa_registers.len(),
                }),
            MemoryKind::ENCODED => {
                let byte = address.index as u8;
                let value = builder.ins().iconst(I8, byte as i64);

                Ok(value)
            }
            _ => Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            }),
        }
    }

    fn get_character(
        &mut self,
        address: Address,
        builder: &mut FunctionBuilder,
    ) -> Result<CraneliftValue, JitError> {
        match address.memory {
            MemoryKind::REGISTER => self
                .ssa_registers
                .get(address.index as usize)
                .map(|ssa_variable| builder.use_var(*ssa_variable))
                .ok_or(JitError::RegisterIndexOutOfBounds {
                    register_index: address.index,
                    total_register_count: self.ssa_registers.len(),
                }),
            MemoryKind::CONSTANT => {
                let character = self.constants.get_character(address.index).ok_or(
                    JitError::ConstantIndexOutOfBounds {
                        constant_index: address.index,
                        total_constant_count: self.constants.len(),
                    },
                )?;
                let value = builder.ins().iconst(I64, character as i64);

                Ok(value)
            }
            _ => Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            }),
        }
    }

    fn get_float(
        &mut self,
        address: Address,
        builder: &mut FunctionBuilder,
    ) -> Result<CraneliftValue, JitError> {
        match address.memory {
            MemoryKind::REGISTER => self
                .ssa_registers
                .get(address.index as usize)
                .map(|ssa_variable| {
                    let i64_value = builder.use_var(*ssa_variable);

                    builder.ins().bitcast(F64, MemFlags::new(), i64_value)
                })
                .ok_or(JitError::RegisterIndexOutOfBounds {
                    register_index: address.index,
                    total_register_count: self.ssa_registers.len(),
                }),
            MemoryKind::CONSTANT => {
                let float = self.constants.get_float(address.index).ok_or(
                    JitError::ConstantIndexOutOfBounds {
                        constant_index: address.index,
                        total_constant_count: self.constants.len(),
                    },
                )?;
                let value = builder.ins().f64const(float);

                Ok(value)
            }
            _ => Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            }),
        }
    }

    fn get_integer(
        &mut self,
        address: Address,
        builder: &mut FunctionBuilder,
    ) -> Result<CraneliftValue, JitError> {
        match address.memory {
            MemoryKind::REGISTER => self
                .ssa_registers
                .get(address.index as usize)
                .map(|ssa_variable| builder.use_var(*ssa_variable))
                .ok_or(JitError::RegisterIndexOutOfBounds {
                    register_index: address.index,
                    total_register_count: self.ssa_registers.len(),
                }),
            MemoryKind::CONSTANT => {
                let integer = self.constants.get_integer(address.index).ok_or(
                    JitError::ConstantIndexOutOfBounds {
                        constant_index: address.index,
                        total_constant_count: self.constants.len(),
                    },
                )?;
                let value = builder.ins().iconst(I64, integer);

                Ok(value)
            }
            _ => Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            }),
        }
    }

    fn get_string(
        &mut self,
        address: Address,
        builder: &mut FunctionBuilder,
    ) -> Result<CraneliftValue, JitError> {
        match address.memory {
            MemoryKind::REGISTER => self
                .ssa_registers
                .get(address.index as usize)
                .map(|ssa_variable| builder.use_var(*ssa_variable))
                .ok_or(JitError::RegisterIndexOutOfBounds {
                    register_index: address.index,
                    total_register_count: self.ssa_registers.len(),
                }),
            MemoryKind::CONSTANT => {
                let string = self.constants.get_string(address.index).ok_or(
                    JitError::ConstantIndexOutOfBounds {
                        constant_index: address.index,
                        total_constant_count: self.constants.len(),
                    },
                )?;

                let allocate_string_function = self.get_allocate_string_function(builder)?;
                let string_pointer = builder
                    .ins()
                    .iconst(self.module.isa().pointer_type(), string.as_ptr() as i64);
                let string_length = builder.ins().iconst(I64, string.len() as i64);
                let call_allocate_string = builder.ins().call(
                    allocate_string_function,
                    &[string_pointer, string_length, self.thread_context],
                );
                let object_pointer = builder.inst_results(call_allocate_string)[0];

                Ok(object_pointer)
            }
            _ => Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            }),
        }
    }

    fn get_prototype_index(
        &self,
        address: Address,
        builder: &mut FunctionBuilder,
    ) -> Result<CraneliftValue, JitError> {
        match address.memory {
            MemoryKind::REGISTER => Ok(self
                .ssa_registers
                .get(address.index as usize)
                .map(|ssa_variable| builder.use_var(*ssa_variable))
                .ok_or(JitError::RegisterIndexOutOfBounds {
                    register_index: address.index,
                    total_register_count: self.ssa_registers.len(),
                })?),
            MemoryKind::CONSTANT => Ok(builder.ins().iconst(I64, address.index as i64)),
            _ => Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            }),
        }
    }

    fn set_register_and_tag(
        &mut self,
        index: u16,
        value: CraneliftValue,
        r#type: OperandType,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let destination_register =
            self.ssa_registers
                .get(index as usize)
                .ok_or(JitError::RegisterIndexOutOfBounds {
                    register_index: index,
                    total_register_count: self.ssa_registers.len(),
                })?;
        let tag = match r#type {
            OperandType::NONE => RegisterTag::EMPTY,
            OperandType::BOOLEAN
            | OperandType::BYTE
            | OperandType::CHARACTER
            | OperandType::FLOAT
            | OperandType::INTEGER
            | OperandType::FUNCTION => RegisterTag::SCALAR,
            OperandType::STRING
            | OperandType::LIST_BOOLEAN
            | OperandType::LIST_BYTE
            | OperandType::LIST_CHARACTER
            | OperandType::LIST_FLOAT
            | OperandType::LIST_INTEGER
            | OperandType::LIST_FUNCTION
            | OperandType::LIST_STRING
            | OperandType::LIST_LIST => RegisterTag::OBJECT,
            _ => {
                return Err(JitError::UnsupportedOperandType {
                    operand_type: r#type,
                });
            }
        };
        let tag_value = builder
            .ins()
            .iconst(RegisterTag::CRANELIFT_TYPE, tag.0 as i64);

        builder.def_var(*destination_register, value);
        self.set_register_tag(index, tag_value, builder)?;

        Ok(())
    }

    fn set_register_tag(
        &mut self,
        index: u16,
        tag_value: CraneliftValue,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let destination_value = builder.ins().iconst(I64, index as i64);
        let absolute_register_index = builder
            .ins()
            .iadd(destination_value, self.base_register_index);
        let tag_offset = builder
            .ins()
            .imul_imm(absolute_register_index, size_of::<RegisterTag>() as i64);
        let tag_address = builder.ins().iadd(
            self.thread_context_fields.register_tag_buffer_pointer,
            tag_offset,
        );

        builder
            .ins()
            .store(MemFlags::new(), tag_value, tag_address, 0);

        Ok(())
    }

    fn declare_imported_function(
        &mut self,
        name: &str,
        signature: Signature,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let function_id = self
            .module
            .declare_function(name, Linkage::Import, &signature)
            .map_err(|error| JitError::CraneliftModuleError {
                error: Box::new(error),
                cranelift_ir: Some(function_builder.func.display().to_string()),
            })?;
        let function_reference = self
            .module
            .declare_func_in_func(function_id, function_builder.func);

        Ok(function_reference)
    }

    fn get_allocate_list_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature.params.extend([
            AbiParam::new(I8),
            AbiParam::new(I64),
            AbiParam::new(pointer_type),
        ]);
        signature.returns.push(AbiParam::new(I64));

        self.declare_imported_function("allocate_list", signature, function_builder)
    }

    fn get_insert_into_list_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature.params.extend([
            AbiParam::new(pointer_type),
            AbiParam::new(I64),
            AbiParam::new(I64),
        ]);

        self.declare_imported_function("insert_into_list", signature, function_builder)
    }

    fn get_get_from_list_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature.params.extend([
            AbiParam::new(I64),
            AbiParam::new(I64),
            AbiParam::new(pointer_type),
        ]);
        signature.returns.push(AbiParam::new(I64));

        self.declare_imported_function("get_from_list", signature, function_builder)
    }

    fn get_compare_lists_equal_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature
            .params
            .extend([AbiParam::new(pointer_type), AbiParam::new(pointer_type)]);
        signature.returns.push(AbiParam::new(I8));

        self.declare_imported_function("compare_lists_equal", signature, function_builder)
    }

    fn get_compare_lists_less_than_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature
            .params
            .extend([AbiParam::new(pointer_type), AbiParam::new(pointer_type)]);
        signature.returns.push(AbiParam::new(I8));

        self.declare_imported_function("compare_lists_less_than", signature, function_builder)
    }

    fn get_compare_lists_less_than_equal_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature
            .params
            .extend([AbiParam::new(pointer_type), AbiParam::new(pointer_type)]);
        signature.returns.push(AbiParam::new(I8));

        self.declare_imported_function("compare_lists_less_than_equal", signature, function_builder)
    }

    fn get_allocate_string_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature.params.extend([
            AbiParam::new(pointer_type),
            AbiParam::new(I64),
            AbiParam::new(pointer_type),
        ]);
        signature.returns.push(AbiParam::new(I64));

        self.declare_imported_function("allocate_string", signature, function_builder)
    }

    fn get_concatenate_strings_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature.params.extend([
            AbiParam::new(pointer_type),
            AbiParam::new(pointer_type),
            AbiParam::new(pointer_type),
        ]);
        signature.returns.push(AbiParam::new(I64));

        self.declare_imported_function("concatenate_strings", signature, function_builder)
    }

    fn get_concatenate_character_string_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature.params.extend([
            AbiParam::new(I64),
            AbiParam::new(pointer_type),
            AbiParam::new(pointer_type),
        ]);
        signature.returns.push(AbiParam::new(I64));

        self.declare_imported_function("concatenate_character_string", signature, function_builder)
    }

    fn get_concatenate_string_character_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature.params.extend([
            AbiParam::new(pointer_type),
            AbiParam::new(I64),
            AbiParam::new(pointer_type),
        ]);
        signature.returns.push(AbiParam::new(I64));

        self.declare_imported_function("concatenate_string_character", signature, function_builder)
    }

    fn get_concatenate_characters_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature.params.extend([
            AbiParam::new(I64),
            AbiParam::new(I64),
            AbiParam::new(pointer_type),
        ]);
        signature.returns.push(AbiParam::new(I64));

        self.declare_imported_function("concatenate_characters", signature, function_builder)
    }

    fn get_compare_strings_equal_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature
            .params
            .extend([AbiParam::new(pointer_type), AbiParam::new(pointer_type)]);
        signature.returns.push(AbiParam::new(I8));

        self.declare_imported_function("compare_strings_equal", signature, function_builder)
    }

    fn get_compare_strings_less_than_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature
            .params
            .extend([AbiParam::new(pointer_type), AbiParam::new(pointer_type)]);
        signature.returns.push(AbiParam::new(I8));

        self.declare_imported_function("compare_strings_less_than", signature, function_builder)
    }

    fn get_compare_strings_less_than_equal_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature
            .params
            .extend([AbiParam::new(pointer_type), AbiParam::new(pointer_type)]);
        signature.returns.push(AbiParam::new(I8));

        self.declare_imported_function(
            "compare_strings_less_than_equal",
            signature,
            function_builder,
        )
    }

    fn get_integer_to_string_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature
            .params
            .extend([AbiParam::new(I64), AbiParam::new(pointer_type)]);
        signature.returns.push(AbiParam::new(I64));

        self.declare_imported_function("integer_to_string", signature, function_builder)
    }

    fn get_read_line_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature.params.push(AbiParam::new(pointer_type));
        signature.returns.push(AbiParam::new(I64));

        self.declare_imported_function("read_line", signature, function_builder)
    }

    fn get_write_line_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature.params.push(AbiParam::new(pointer_type));

        self.declare_imported_function("write_line", signature, function_builder)
    }

    fn get_byte_power_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature
            .params
            .extend([AbiParam::new(I8), AbiParam::new(I8)]);
        signature.returns.push(AbiParam::new(I64));

        self.declare_imported_function("byte_power", signature, function_builder)
    }

    fn get_integer_power_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature
            .params
            .extend([AbiParam::new(I64), AbiParam::new(I64)]);
        signature.returns.push(AbiParam::new(I64));

        self.declare_imported_function("integer_power", signature, function_builder)
    }

    fn get_float_power_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature
            .params
            .extend([AbiParam::new(F64), AbiParam::new(F64)]);
        signature.returns.push(AbiParam::new(I64));

        self.declare_imported_function("float_power", signature, function_builder)
    }

    #[cfg(debug_assertions)]
    fn get_log_operation_and_ip_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
    ) -> Result<FuncRef, JitError> {
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        signature.params.push(AbiParam::new(I8));
        signature.params.push(AbiParam::new(I64));

        self.declare_imported_function("log_operation_and_ip", signature, function_builder)
    }
}
