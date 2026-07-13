use crate::{
    constants::Constants,
    instruction::{Address, Instruction, MemoryKind, Move, OperandType, Operation},
    prototype::Prototype,
    vm::{error::VmError, register::Register},
};

pub struct Call<'a> {
    instruction_pointer: usize,
    prototype: &'a Prototype,
    registers: &'a mut [Register],
    constants: &'a Constants,
    call_stack: &'a mut Vec<CallFrame>,
}

impl<'a> Call<'a> {
    pub fn new(
        instruction_pointer: usize,
        prototype: &'a Prototype,
        registers: &'a mut [Register],
        constants: &'a Constants,
        call_stack: &'a mut Vec<CallFrame>,
    ) -> Result<Self, VmError> {
        Ok(Self {
            instruction_pointer,
            prototype,
            registers,
            constants,
            call_stack,
        })
    }

    pub fn run(&mut self) -> Result<Option<Vec<Register>>, VmError> {
        loop {
            let instruction = self.prototype.instructions[self.instruction_pointer];
            let operation = instruction.operation();

            match operation {
                Operation::MOVE => self.r#move(instruction)?,
                Operation::RETURN => {
                    self.call_stack.pop();

                    if self.call_stack.is_empty() {
                        let return_register_count =
                            self.prototype
                                .return_types
                                .iter()
                                .fold(0, |previous, operand_type| {
                                    previous + operand_type.register_width().as_u16()
                                }) as usize;
                        let return_registers_start = self.registers.len() - return_register_count;
                        let return_registers = self.registers[return_registers_start..].to_vec();

                        return Ok(Some(return_registers));
                    } else {
                        return Ok(None);
                    }
                }
                _ => return Err(VmError::UnsupportedOperation { operation }),
            }
        }
    }

    fn r#move(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let Move {
            destination,
            operand_type,
            operand:
                Address {
                    memory: operand_memory,
                    index: operand_index,
                },
            jump_distance,
            jump_forward,
        } = Move::from(instruction);

        match operand_type {
            OperandType::I_32 => {
                let register_value = match operand_memory {
                    MemoryKind::CONSTANT => self.constants.get_i32(operand_index)? as u32,
                    MemoryKind::REGISTER => self.registers[operand_index as usize].0,
                    MemoryKind::ENCODED => operand_index as u32,
                    _ => {
                        return Err(VmError::UnsupportedMemoryKind {
                            memory: operand_memory,
                        });
                    }
                };

                self.registers[destination as usize] = Register(register_value);
            }
            OperandType::F_64 => {
                let (low_bits, high_bits) = match operand_memory {
                    MemoryKind::CONSTANT => {
                        let bits = self.constants.get_f64(operand_index)?.to_bits();

                        (bits as u32, (bits >> 32) as u32)
                    }
                    MemoryKind::REGISTER => {
                        let register_value = self.registers[operand_index as usize].0;
                        let next_register_value = self.registers[(operand_index + 1) as usize].0;

                        (register_value, next_register_value)
                    }
                    _ => {
                        return Err(VmError::UnsupportedMemoryKind {
                            memory: operand_memory,
                        });
                    }
                };
                let destination = destination as usize;
                let next_destination = destination + 1;

                self.registers[destination] = Register(low_bits);
                self.registers[next_destination] = Register(high_bits);
            }
            _ => {
                return Err(VmError::UnsupportedOperandType { operand_type });
            }
        }

        if jump_distance > 0 {
            if jump_forward {
                self.instruction_pointer += jump_distance as usize;
            } else {
                self.instruction_pointer -= jump_distance as usize;
            }
        } else {
            self.instruction_pointer += 1;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CallFrame {
    pub prototype_id: u16,
    pub regsiter_range_start: u16,
    pub register_range_end: u16,
    pub instruction_pointer: usize,
}
