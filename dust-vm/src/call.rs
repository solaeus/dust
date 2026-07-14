use dust_compiler::{
    constants::Constants,
    instruction::{Add, Address, Instruction, MemoryKind, Move, OperandType, Operation},
    prototype::Prototype,
};
use tracing::debug;

use crate::{error::VmError, register::Register};

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

    pub fn run(mut self) -> Result<Option<Vec<Register>>, VmError> {
        loop {
            let instruction = self.prototype.instructions[self.instruction_pointer];
            let operation = instruction.operation();

            debug!(
                "Call at IP {}: {operation}, registers: {:?}",
                self.instruction_pointer, self.registers
            );

            match operation {
                Operation::MOVE => self.r#move(instruction)?,
                Operation::ADD => self.add(instruction)?,
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
                        let return_registers = self.registers[..return_register_count].to_vec();

                        return Ok(Some(return_registers));
                    } else {
                        return Ok(None);
                    }
                }
                _ => return Err(VmError::UnsupportedOperation { operation }),
            }
        }
    }

    fn get_i8(&self, Address { memory, index }: Address) -> Result<i8, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as i8),
            MemoryKind::REGISTER => Ok(self.registers[index as usize].0 as i8),
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_i16(&self, Address { memory, index }: Address) -> Result<i16, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as i16),
            MemoryKind::REGISTER => Ok(self.registers[index as usize].0 as i16),
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_i32(&self, Address { memory, index }: Address) -> Result<i32, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as i32),
            MemoryKind::CONSTANT => Ok(self.constants.get_i32(index)?),
            MemoryKind::REGISTER => Ok(self.registers[index as usize].0 as i32),
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_i64(&self, Address { memory, index }: Address) -> Result<i64, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as i64),
            MemoryKind::CONSTANT => Ok(self.constants.get_i64(index)?),
            MemoryKind::REGISTER => {
                let low_bits = self.registers[index as usize].0 as i64;
                let high_bits = self.registers[index as usize + 1].0 as i64;

                Ok((high_bits << 32) | low_bits)
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_i128(&self, Address { memory, index }: Address) -> Result<i128, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as i128),
            MemoryKind::CONSTANT => Ok(self.constants.get_i128(index)?),
            MemoryKind::REGISTER => {
                let low_bits = self.registers[index as usize].0 as i128;
                let low_mid_bits = self.registers[index as usize + 1].0 as i128;
                let high_mid_bits = self.registers[index as usize + 2].0 as i128;
                let high_bits = self.registers[index as usize + 3].0 as i128;
                let full_bits =
                    high_bits << 96 | high_mid_bits << 64 | low_mid_bits << 32 | low_bits;

                Ok(full_bits)
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_u8(&self, Address { memory, index }: Address) -> Result<u8, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as u8),
            MemoryKind::REGISTER => Ok(self.registers[index as usize].0 as u8),
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_u16(&self, Address { memory, index }: Address) -> Result<u16, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index),
            MemoryKind::REGISTER => Ok(self.registers[index as usize].0 as u16),
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_u32(&self, Address { memory, index }: Address) -> Result<u32, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as u32),
            MemoryKind::CONSTANT => Ok(self.constants.get_u32(index)?),
            MemoryKind::REGISTER => Ok(self.registers[index as usize].0),
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_u64(&self, Address { memory, index }: Address) -> Result<u64, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as u64),
            MemoryKind::CONSTANT => Ok(self.constants.get_u64(index)?),
            MemoryKind::REGISTER => {
                let low_bits = self.registers[index as usize].0 as u64;
                let high_bits = self.registers[index as usize + 1].0 as u64;

                Ok((high_bits << 32) | low_bits)
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_u128(&self, Address { memory, index }: Address) -> Result<u128, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as u128),
            MemoryKind::CONSTANT => Ok(self.constants.get_u128(index)?),
            MemoryKind::REGISTER => {
                let low_bits = self.registers[index as usize].0 as u128;
                let low_mid_bits = self.registers[index as usize + 1].0 as u128;
                let high_mid_bits = self.registers[index as usize + 2].0 as u128;
                let high_bits = self.registers[index as usize + 3].0 as u128;
                let full_bits =
                    high_bits << 96 | high_mid_bits << 64 | low_mid_bits << 32 | low_bits;

                Ok(full_bits)
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_f32(&self, Address { memory, index }: Address) -> Result<f32, VmError> {
        match memory {
            MemoryKind::CONSTANT => Ok(self.constants.get_f32(index)?),
            MemoryKind::REGISTER => {
                let low_bits = self.registers[index as usize].0;
                let high_bits = self.registers[index as usize + 1].0;
                let full_bits = high_bits | low_bits;

                Ok(f32::from_bits(full_bits))
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_f64(&self, Address { memory, index }: Address) -> Result<f64, VmError> {
        match memory {
            MemoryKind::CONSTANT => Ok(self.constants.get_f64(index)?),
            MemoryKind::REGISTER => {
                let low_bits = self.registers[index as usize].0;
                let high_bits = self.registers[index as usize + 1].0;
                let full_bits = (high_bits as u64) << 32 | low_bits as u64;

                Ok(f64::from_bits(full_bits))
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn set_register(&mut self, destination: u16, register: Register) -> Result<(), VmError> {
        if destination as usize >= self.registers.len() {
            return Err(VmError::InvalidRegisterIndex { index: destination });
        }

        self.registers[destination as usize] = register;

        Ok(())
    }

    fn set_double_registers(
        &mut self,
        destination: u16,
        low: Register,
        high: Register,
    ) -> Result<(), VmError> {
        if destination as usize + 1 >= self.registers.len() {
            return Err(VmError::InvalidRegisterIndex { index: destination });
        }

        self.registers[destination as usize] = high;
        self.registers[destination as usize + 1] = low;

        Ok(())
    }

    fn set_quad_registers(
        &mut self,
        destination: u16,
        low: Register,
        low_mid: Register,
        high_mid: Register,
        high: Register,
    ) -> Result<(), VmError> {
        if destination as usize + 3 >= self.registers.len() {
            return Err(VmError::InvalidRegisterIndex { index: destination });
        }

        self.registers[destination as usize] = high;
        self.registers[destination as usize + 1] = high_mid;
        self.registers[destination as usize + 2] = low_mid;
        self.registers[destination as usize + 3] = low;

        Ok(())
    }

    fn set_i64_to_registers(&mut self, value: i64, destination: u16) -> Result<(), VmError> {
        let low_bits = (value & 0xFFFFFFFF) as u32;
        let high_bits = ((value >> 32) & 0xFFFFFFFF) as u32;

        self.set_double_registers(destination, Register(low_bits), Register(high_bits))
    }

    fn set_i128_to_registers(&mut self, value: i128, destination: u16) -> Result<(), VmError> {
        let low_bits = (value & 0xFFFFFFFF) as u32;
        let low_mid_bits = ((value >> 32) & 0xFFFFFFFF) as u32;
        let high_mid_bits = ((value >> 64) & 0xFFFFFFFF) as u32;
        let high_bits = ((value >> 96) & 0xFFFFFFFF) as u32;

        self.set_quad_registers(
            destination,
            Register(low_bits),
            Register(low_mid_bits),
            Register(high_mid_bits),
            Register(high_bits),
        )
    }

    fn set_u64_to_registers(&mut self, value: u64, destination: u16) -> Result<(), VmError> {
        let low_bits = (value & 0xFFFFFFFF) as u32;
        let high_bits = ((value >> 32) & 0xFFFFFFFF) as u32;

        self.set_double_registers(destination, Register(low_bits), Register(high_bits))
    }

    fn set_u128_to_registers(&mut self, value: u128, destination: u16) -> Result<(), VmError> {
        let low_bits = (value & 0xFFFFFFFF) as u32;
        let low_mid_bits = ((value >> 32) & 0xFFFFFFFF) as u32;
        let high_mid_bits = ((value >> 64) & 0xFFFFFFFF) as u32;
        let high_bits = ((value >> 96) & 0xFFFFFFFF) as u32;

        self.set_quad_registers(
            destination,
            Register(low_bits),
            Register(low_mid_bits),
            Register(high_mid_bits),
            Register(high_bits),
        )
    }

    fn set_f64_to_registers(&mut self, value: f64, destination: u16) -> Result<(), VmError> {
        let bits = value.to_bits();
        let low_bits = (bits & 0xFFFFFFFF) as u32;
        let high_bits = (bits >> 32 & 0xFFFFFFFF) as u32;

        self.set_double_registers(destination, Register(low_bits), Register(high_bits))
    }

    fn copy_registers_to_registers(
        &mut self,
        operand_index: u16,
        operand_type: OperandType,
        destination: u16,
    ) -> Result<(), VmError> {
        let operand_index = operand_index as usize;
        let count = operand_type.register_width().as_usize();
        let operand_range = operand_index..operand_index + count;
        let destination = destination as usize;

        if operand_range.end > self.registers.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: operand_index as u16,
            });
        }

        if destination + count >= self.registers.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: destination as u16,
            });
        }

        self.registers.copy_within(operand_range, destination);

        Ok(())
    }

    fn copy_encoded_to_register(
        &mut self,
        operand_index: u16,
        operand_type: OperandType,
        destination: u16,
    ) -> Result<(), VmError> {
        let register = match operand_type {
            OperandType::BOOLEAN => Register((operand_index != 0) as u32),
            OperandType::I_8
            | OperandType::I_16
            | OperandType::I_32
            | OperandType::I_64
            | OperandType::I_128
            | OperandType::U_8
            | OperandType::U_16
            | OperandType::U_32
            | OperandType::U_64
            | OperandType::U_128
            | OperandType::F_32
            | OperandType::F_64
            | OperandType::CHARACTER
            | OperandType::FUNCTION => Register(operand_index as u32),
            _ => return Err(VmError::UnsupportedOperandType { operand_type }),
        };
        self.registers[destination as usize] = register;

        Ok(())
    }

    fn copy_constant_to_registers(
        &mut self,
        operand_index: u16,
        operand_type: OperandType,
        destination: u16,
    ) -> Result<(), VmError> {
        match operand_type {
            OperandType::I_32 => {
                let constant = self.constants.get_i32(operand_index)?;

                self.set_register(destination, Register(constant as u32))?;
            }
            OperandType::I_64 => {
                let constant = self.constants.get_i64(operand_index)?;

                self.set_i64_to_registers(constant, destination)?;
            }
            OperandType::I_128 => {
                let constant = self.constants.get_i128(operand_index)?;

                self.set_i128_to_registers(constant, destination)?;
            }
            OperandType::U_32 => {
                let constant = self.constants.get_u32(operand_index)?;

                self.set_register(destination, Register(constant))?;
            }
            OperandType::U_64 => {
                let constant = self.constants.get_u64(operand_index)?;

                self.set_u64_to_registers(constant, destination)?;
            }
            OperandType::U_128 => {
                let constant = self.constants.get_u128(operand_index)?;

                self.set_u128_to_registers(constant, destination)?;
            }
            OperandType::F_32 => {
                let constant = self.constants.get_f32(operand_index)?;

                self.set_register(destination, Register(constant.to_bits()))?;
            }
            OperandType::F_64 => {
                let constant = self.constants.get_f64(operand_index)?;

                self.set_f64_to_registers(constant, destination)?;
            }
            OperandType::CHARACTER => {
                let constant = self.constants.get_character(operand_index)?;

                self.set_register(destination, Register(constant as u32))?;
            }
            _ => return Err(VmError::UnsupportedOperandType { operand_type }),
        }

        Ok(())
    }

    fn r#move(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let Move {
            destination,
            operand_type,
            operand,
            jump_distance,
            jump_forward,
        } = Move::from(instruction);

        match operand.memory {
            MemoryKind::ENCODED => {
                self.copy_encoded_to_register(operand.index, operand_type, destination)?;
            }
            MemoryKind::REGISTER => {
                self.copy_registers_to_registers(operand.index, operand_type, destination)?;
            }
            MemoryKind::CONSTANT => {
                self.copy_constant_to_registers(operand.index, operand_type, destination)?;
            }
            _ => {
                return Err(VmError::UnsupportedMemoryKind {
                    memory: operand.memory,
                });
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

    fn add(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let Add {
            destination,
            operand_type,
            left_address,
            right_address,
        } = Add::from(instruction);

        match operand_type {
            OperandType::I_8 => {
                let left = self.get_i8(left_address)?;
                let right = self.get_i8(right_address)?;
                let sum = left + right;

                self.set_register(destination, Register(sum as u32))?;
            }
            OperandType::I_16 => {
                let left = self.get_i16(left_address)?;
                let right = self.get_i16(right_address)?;
                let sum = left + right;

                self.set_register(destination, Register(sum as u32))?;
            }
            OperandType::I_32 => {
                let left = self.get_i32(left_address)?;
                let right = self.get_i32(right_address)?;
                let sum = left + right;

                self.set_register(destination, Register(sum as u32))?;
            }
            OperandType::I_64 => {
                let left = self.get_i64(left_address)?;
                let right = self.get_i64(right_address)?;
                let sum = left + right;

                self.set_i64_to_registers(sum, destination)?;
            }
            OperandType::I_128 => {
                let left = self.get_i128(left_address)?;
                let right = self.get_i128(right_address)?;
                let sum = left + right;

                self.set_i128_to_registers(sum, destination)?;
            }
            OperandType::U_8 => {
                let left = self.get_u8(left_address)?;
                let right = self.get_u8(right_address)?;
                let sum = left + right;

                self.set_register(destination, Register(sum as u32))?;
            }
            OperandType::U_16 => {
                let left = self.get_u16(left_address)?;
                let right = self.get_u16(right_address)?;
                let sum = left + right;

                self.set_register(destination, Register(sum as u32))?;
            }
            OperandType::U_32 => {
                let left = self.get_u32(left_address)?;
                let right = self.get_u32(right_address)?;
                let sum = left + right;

                self.set_register(destination, Register(sum))?;
            }
            OperandType::U_64 => {
                let left = self.get_u64(left_address)?;
                let right = self.get_u64(right_address)?;
                let sum = left + right;

                self.set_u64_to_registers(sum, destination)?;
            }
            OperandType::U_128 => {
                let left = self.get_u128(left_address)?;
                let right = self.get_u128(right_address)?;
                let sum = left + right;

                self.set_u128_to_registers(sum, destination)?;
            }
            OperandType::F_32 => {
                let left = self.get_f32(left_address)?;
                let right = self.get_f32(right_address)?;
                let sum = left + right;

                self.set_register(destination, Register(sum.to_bits()))?;
            }
            OperandType::F_64 => {
                let left = self.get_f64(left_address)?;
                let right = self.get_f64(right_address)?;
                let sum = left + right;
                let bits = sum.to_bits();
                let low_bits = bits as u32;
                let high_bits = (bits >> 32) as u32;

                self.set_double_registers(destination, Register(low_bits), Register(high_bits))?;
            }
            _ => return Err(VmError::UnsupportedOperandType { operand_type }),
        }

        self.instruction_pointer += 1;

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
