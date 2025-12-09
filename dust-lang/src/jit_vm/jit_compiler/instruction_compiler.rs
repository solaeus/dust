use std::mem::offset_of;

use cranelift::prelude::{
    Block, FunctionBuilder, InstBuilder, MemFlags, Signature, Type as CraneliftType,
    Value as CraneliftValue, Variable,
    types::{I8, I64},
};

use crate::{
    instruction::{Address, Instruction, MemoryKind, Move, OperandType, Operation, Return},
    jit_vm::{JitError, RegisterTag, jit_compiler::ExecutionSegment, thread::ThreadContext},
};

pub struct InstructionCompiler<'a> {
    pub is_entry_segment: bool,
    pub instruction_blocks: &'a [Block],
    pub ssa_registers: &'a mut [Variable],
    pub register_tags_buffer_pointer: CraneliftValue,
    pub base_register_index: CraneliftValue,
    pub continuation_function: CraneliftValue,
    pub thread_context: CraneliftValue,
    pub pointer_type: CraneliftType,
    pub continuation_signature: &'a Signature,
}

impl<'a> InstructionCompiler<'a> {
    pub fn compile(
        &mut self,
        instruction: &Instruction,
        ip: usize,
        block_index: usize,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        builder.switch_to_block(self.instruction_blocks[block_index]);

        let operation = instruction.operation();

        match operation {
            Operation::MOVE => self.compile_move_instruction(instruction, ip, builder),
            Operation::RETURN => self.compile_return_instruction(instruction, builder),
            _ => Err(JitError::UnsupportedOperation { operation }),
        }
    }

    fn compile_move_instruction(
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

        self.set_register(destination, operand_value, r#type, builder)?;
        self.set_register_tag(destination, r#type, builder)?;

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

    fn compile_return_instruction(
        &mut self,
        instruction: &Instruction,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let Return {
            should_return_value: _,
            return_value_address,
            r#type,
        } = Return::from(instruction);

        let return_value = if r#type == OperandType::NONE {
            builder.ins().iconst(I64, 0)
        } else {
            self.get_value(return_value_address, r#type, builder)?
        };

        builder.ins().store(
            MemFlags::new(),
            self.base_register_index,
            self.thread_context,
            offset_of!(ThreadContext, registers_used) as i32,
        );

        if self.is_entry_segment {
            builder.ins().return_(&[return_value]);
        } else {
            let continuation_signature = builder
                .func
                .import_signature(self.continuation_signature.clone());

            builder.ins().return_call_indirect(
                continuation_signature,
                self.continuation_function,
                &[self.thread_context, return_value],
            );
        }

        Ok(())
    }

    fn get_value(
        &mut self,
        address: Address,
        r#type: OperandType,
        builder: &mut FunctionBuilder,
    ) -> Result<CraneliftValue, JitError> {
        match r#type {
            OperandType::BOOLEAN => self.get_boolean(address, builder),
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
            MemoryKind::REGISTER => {
                if let Some(ssa_variable) = self.ssa_registers.get(address.index as usize) {
                    Ok(builder.use_var(*ssa_variable))
                } else {
                    Err(JitError::RegisterIndexOutOfBounds {
                        register_index: address.index,
                        total_register_count: self.ssa_registers.len(),
                    })
                }
            }
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

    fn set_register(
        &mut self,
        destination: u16,
        value: CraneliftValue,
        r#type: OperandType,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let destination_register = self.ssa_registers.get(destination as usize).ok_or(
            JitError::RegisterIndexOutOfBounds {
                register_index: destination,
                total_register_count: self.ssa_registers.len(),
            },
        )?;

        builder.def_var(*destination_register, value);
        self.set_register_tag(destination, r#type, builder)?;

        Ok(())
    }

    fn set_register_tag(
        &mut self,
        destination: u16,
        r#type: OperandType,
        builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let tag = match r#type {
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
        let destination_value = builder
            .ins()
            .iconst(RegisterTag::CRANELIFT_TYPE, destination as i64);
        let absolute_register_index = builder
            .ins()
            .iadd(destination_value, self.base_register_index);
        let tag_offset = builder
            .ins()
            .imul_imm(absolute_register_index, size_of::<RegisterTag>() as i64);
        let tag_address = builder
            .ins()
            .iadd(self.register_tags_buffer_pointer, tag_offset);

        builder
            .ins()
            .store(MemFlags::new(), tag_value, tag_address, 0);

        Ok(())
    }
}
