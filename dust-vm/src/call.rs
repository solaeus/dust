use dust_compiler::{
    constants::Constants,
    instruction::{
        Add, Address, Divide, Exponent, Instruction, MemoryKind, Modulo, Move, Multiply,
        OperandType, Operation, Subtract,
    },
    prototype::Prototype,
};
use tracing::debug;

use crate::{error::VmError, register::Register};

pub struct Call<'a> {
    frame: CallFrame,

    prototypes: &'a [Prototype],

    register_stack: &'a mut [Register],

    call_stack: &'a mut Vec<CallFrame>,

    constants: &'a Constants,
}

impl<'a> Call<'a> {
    pub fn new(
        prototypes: &'a [Prototype],
        register_stack: &'a mut [Register],
        constants: &'a Constants,
        call_stack: &'a mut Vec<CallFrame>,
    ) -> Result<Self, VmError> {
        let frame = call_stack
            .last()
            .cloned()
            .ok_or(VmError::CallStackUnderflow)?;

        Ok(Self {
            frame,
            prototypes,
            register_stack,
            constants,
            call_stack,
        })
    }

    pub fn run(mut self) -> Result<Option<Vec<Register>>, VmError> {
        let prototype = self.prototypes.get(self.frame.prototype_index).ok_or(
            VmError::InvalidPrototypeIndex {
                index: self.frame.prototype_index,
            },
        )?;

        loop {
            let instruction = *prototype
                .instructions
                .get(self.frame.instruction_pointer)
                .ok_or(VmError::InvalidInstructionPointer {
                    index: self.frame.instruction_pointer,
                })?;
            let operation = instruction.operation();

            debug!(
                "Call at IP {}: {operation}, registers: {:?}",
                self.frame.instruction_pointer, self.register_stack
            );

            match operation {
                Operation::MOVE => self.r#move(instruction)?,
                Operation::ADD => self.add(instruction)?,
                Operation::SUBTRACT => self.subtract(instruction)?,
                Operation::MULTIPLY => self.multiply(instruction)?,
                Operation::DIVIDE => self.divide(instruction)?,
                Operation::MODULO => self.modulo(instruction)?,
                Operation::EXPONENT => self.exponent(instruction)?,
                Operation::RETURN => {
                    self.call_stack.pop();

                    if self.call_stack.is_empty() {
                        let return_register_count =
                            prototype
                                .return_types
                                .iter()
                                .fold(0, |previous, operand_type| {
                                    previous + operand_type.register_width().as_u16()
                                }) as usize;
                        let return_registers =
                            self.register_stack[..return_register_count].to_vec();

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
                self.frame.instruction_pointer += jump_distance as usize;
            } else {
                self.frame.instruction_pointer -= jump_distance as usize;
            }
        } else {
            self.frame.instruction_pointer += 1;
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

                self.set_register(destination, Register::new(sum))?;
            }
            OperandType::I_16 => {
                let left = self.get_i16(left_address)?;
                let right = self.get_i16(right_address)?;
                let sum = left + right;

                self.set_register(destination, Register::new(sum))?;
            }
            OperandType::I_32 => {
                let left = self.get_i32(left_address)?;
                let right = self.get_i32(right_address)?;
                let sum = left + right;

                self.set_register(destination, Register::new(sum))?;
            }
            OperandType::I_64 => {
                let left = self.get_i64(left_address)?;
                let right = self.get_i64(right_address)?;
                let sum = left + right;

                self.set_registers_to_i64(sum, destination)?;
            }
            OperandType::I_128 => {
                let left = self.get_i128(left_address)?;
                let right = self.get_i128(right_address)?;
                let sum = left + right;

                self.set_registers_to_i128(sum, destination)?;
            }
            OperandType::U_8 => {
                let left = self.get_u8(left_address)?;
                let right = self.get_u8(right_address)?;
                let sum = left + right;

                self.set_register(destination, Register::new(sum))?;
            }
            OperandType::U_16 => {
                let left = self.get_u16(left_address)?;
                let right = self.get_u16(right_address)?;
                let sum = left + right;

                self.set_register(destination, Register::new(sum))?;
            }
            OperandType::U_32 => {
                let left = self.get_u32(left_address)?;
                let right = self.get_u32(right_address)?;
                let sum = left + right;

                self.set_register(destination, Register::new(sum))?;
            }
            OperandType::U_64 => {
                let left = self.get_u64(left_address)?;
                let right = self.get_u64(right_address)?;
                let sum = left + right;

                self.set_registers_to_u64(sum, destination)?;
            }
            OperandType::U_128 => {
                let left = self.get_u128(left_address)?;
                let right = self.get_u128(right_address)?;
                let sum = left + right;

                self.set_registers_to_u128(sum, destination)?;
            }
            OperandType::F_32 => {
                let left = self.get_f32(left_address)?;
                let right = self.get_f32(right_address)?;
                let sum = left + right;

                self.set_register(destination, Register::new(sum))?;
            }
            OperandType::F_64 => {
                let left = self.get_f64(left_address)?;
                let right = self.get_f64(right_address)?;
                let sum = left + right;

                self.set_registers_to_f64(sum, destination)?;
            }
            _ => return Err(VmError::UnsupportedOperandType { operand_type }),
        }

        self.frame.instruction_pointer += 1;

        Ok(())
    }

    fn subtract(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let Subtract {
            destination,
            operand_type,
            left_address,
            right_address,
        } = Subtract::from(instruction);

        match operand_type {
            OperandType::I_8 => {
                let left = self.get_i8(left_address)?;
                let right = self.get_i8(right_address)?;
                let difference = left - right;

                self.set_register(destination, Register::new(difference as u32))?;
            }
            OperandType::I_16 => {
                let left = self.get_i16(left_address)?;
                let right = self.get_i16(right_address)?;
                let difference = left - right;

                self.set_register(destination, Register::new(difference))?;
            }
            OperandType::I_32 => {
                let left = self.get_i32(left_address)?;
                let right = self.get_i32(right_address)?;
                let difference = left - right;

                self.set_register(destination, Register::new(difference))?;
            }
            OperandType::I_64 => {
                let left = self.get_i64(left_address)?;
                let right = self.get_i64(right_address)?;
                let difference = left - right;

                self.set_registers_to_i64(difference, destination)?;
            }
            OperandType::I_128 => {
                let left = self.get_i128(left_address)?;
                let right = self.get_i128(right_address)?;
                let difference = left - right;

                self.set_registers_to_i128(difference, destination)?;
            }
            OperandType::U_8 => {
                let left = self.get_u8(left_address)?;
                let right = self.get_u8(right_address)?;
                let difference = left - right;

                self.set_register(destination, Register::new(difference))?;
            }
            OperandType::U_16 => {
                let left = self.get_u16(left_address)?;
                let right = self.get_u16(right_address)?;
                let difference = left - right;

                self.set_register(destination, Register::new(difference))?;
            }
            OperandType::U_32 => {
                let left = self.get_u32(left_address)?;
                let right = self.get_u32(right_address)?;
                let difference = left - right;

                self.set_register(destination, Register::new(difference))?;
            }
            OperandType::U_64 => {
                let left = self.get_u64(left_address)?;
                let right = self.get_u64(right_address)?;
                let difference = left - right;

                self.set_registers_to_u64(difference, destination)?;
            }
            OperandType::U_128 => {
                let left = self.get_u128(left_address)?;
                let right = self.get_u128(right_address)?;
                let difference = left - right;

                self.set_registers_to_u128(difference, destination)?;
            }
            OperandType::F_32 => {
                let left = self.get_f32(left_address)?;
                let right = self.get_f32(right_address)?;
                let difference = left - right;

                self.set_register(destination, Register::new(difference.to_bits()))?;
            }
            OperandType::F_64 => {
                let left = self.get_f64(left_address)?;
                let right = self.get_f64(right_address)?;
                let difference = left - right;

                self.set_registers_to_f64(difference, destination)?;
            }
            _ => return Err(VmError::UnsupportedOperandType { operand_type }),
        }

        self.frame.instruction_pointer += 1;

        Ok(())
    }

    fn multiply(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let Multiply {
            destination,
            operand_type,
            left_address,
            right_address,
        } = Multiply::from(instruction);

        match operand_type {
            OperandType::I_8 => {
                let left = self.get_i8(left_address)?;
                let right = self.get_i8(right_address)?;
                let product = left * right;

                self.set_register(destination, Register::new(product))?;
            }
            OperandType::I_16 => {
                let left = self.get_i16(left_address)?;
                let right = self.get_i16(right_address)?;
                let product = left * right;

                self.set_register(destination, Register::new(product))?;
            }
            OperandType::I_32 => {
                let left = self.get_i32(left_address)?;
                let right = self.get_i32(right_address)?;
                let product = left * right;

                self.set_register(destination, Register::new(product))?;
            }
            OperandType::I_64 => {
                let left = self.get_i64(left_address)?;
                let right = self.get_i64(right_address)?;
                let product = left * right;

                self.set_registers_to_i64(product, destination)?;
            }
            OperandType::I_128 => {
                let left = self.get_i128(left_address)?;
                let right = self.get_i128(right_address)?;
                let product = left * right;

                self.set_registers_to_i128(product, destination)?;
            }
            OperandType::U_8 => {
                let left = self.get_u8(left_address)?;
                let right = self.get_u8(right_address)?;
                let product = left * right;

                self.set_register(destination, Register::new(product))?;
            }
            OperandType::U_16 => {
                let left = self.get_u16(left_address)?;
                let right = self.get_u16(right_address)?;
                let product = left * right;

                self.set_register(destination, Register::new(product))?;
            }
            OperandType::U_32 => {
                let left = self.get_u32(left_address)?;
                let right = self.get_u32(right_address)?;
                let product = left * right;

                self.set_register(destination, Register::new(product))?;
            }
            OperandType::U_64 => {
                let left = self.get_u64(left_address)?;
                let right = self.get_u64(right_address)?;
                let product = left * right;

                self.set_registers_to_u64(product, destination)?;
            }
            OperandType::U_128 => {
                let left = self.get_u128(left_address)?;
                let right = self.get_u128(right_address)?;
                let product = left * right;

                self.set_registers_to_u128(product, destination)?;
            }
            OperandType::F_32 => {
                let left = self.get_f32(left_address)?;
                let right = self.get_f32(right_address)?;
                let product = left * right;

                self.set_register(destination, Register::new(product))?;
            }
            OperandType::F_64 => {
                let left = self.get_f64(left_address)?;
                let right = self.get_f64(right_address)?;
                let product = left * right;

                self.set_registers_to_f64(product, destination)?;
            }
            _ => return Err(VmError::UnsupportedOperandType { operand_type }),
        }

        self.frame.instruction_pointer += 1;

        Ok(())
    }

    fn divide(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let Divide {
            destination,
            operand_type,
            left_address,
            right_address,
        } = Divide::from(instruction);

        match operand_type {
            OperandType::I_8 => {
                let left = self.get_i8(left_address)?;
                let right = self.get_i8(right_address)?;
                let quotient = left / right;

                self.set_register(destination, Register::new(quotient))?;
            }
            OperandType::I_16 => {
                let left = self.get_i16(left_address)?;
                let right = self.get_i16(right_address)?;
                let quotient = left / right;

                self.set_register(destination, Register::new(quotient))?;
            }
            OperandType::I_32 => {
                let left = self.get_i32(left_address)?;
                let right = self.get_i32(right_address)?;
                let quotient = left / right;

                self.set_register(destination, Register::new(quotient))?;
            }
            OperandType::I_64 => {
                let left = self.get_i64(left_address)?;
                let right = self.get_i64(right_address)?;
                let quotient = left / right;

                self.set_registers_to_i64(quotient, destination)?;
            }
            OperandType::I_128 => {
                let left = self.get_i128(left_address)?;
                let right = self.get_i128(right_address)?;
                let quotient = left / right;

                self.set_registers_to_i128(quotient, destination)?;
            }
            OperandType::U_8 => {
                let left = self.get_u8(left_address)?;
                let right = self.get_u8(right_address)?;
                let quotient = left / right;

                self.set_register(destination, Register::new(quotient))?;
            }
            OperandType::U_16 => {
                let left = self.get_u16(left_address)?;
                let right = self.get_u16(right_address)?;
                let quotient = left / right;

                self.set_register(destination, Register::new(quotient))?;
            }
            OperandType::U_32 => {
                let left = self.get_u32(left_address)?;
                let right = self.get_u32(right_address)?;
                let quotient = left / right;

                self.set_register(destination, Register::new(quotient))?;
            }
            OperandType::U_64 => {
                let left = self.get_u64(left_address)?;
                let right = self.get_u64(right_address)?;
                let quotient = left / right;

                self.set_registers_to_u64(quotient, destination)?;
            }
            OperandType::U_128 => {
                let left = self.get_u128(left_address)?;
                let right = self.get_u128(right_address)?;
                let quotient = left / right;

                self.set_registers_to_u128(quotient, destination)?;
            }
            OperandType::F_32 => {
                let left = self.get_f32(left_address)?;
                let right = self.get_f32(right_address)?;
                let quotient = left / right;

                self.set_register(destination, Register::new(quotient))?;
            }
            OperandType::F_64 => {
                let left = self.get_f64(left_address)?;
                let right = self.get_f64(right_address)?;
                let quotient = left / right;

                self.set_registers_to_f64(quotient, destination)?;
            }
            _ => return Err(VmError::UnsupportedOperandType { operand_type }),
        }

        self.frame.instruction_pointer += 1;

        Ok(())
    }

    fn modulo(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let Modulo {
            destination,
            operand_type,
            left_address,
            right_address,
        } = Modulo::from(instruction);

        match operand_type {
            OperandType::I_8 => {
                let left = self.get_i8(left_address)?;
                let right = self.get_i8(right_address)?;
                let remainder = left % right;

                self.set_register(destination, Register::new(remainder))?;
            }
            OperandType::I_16 => {
                let left = self.get_i16(left_address)?;
                let right = self.get_i16(right_address)?;
                let remainder = left % right;

                self.set_register(destination, Register::new(remainder))?;
            }
            OperandType::I_32 => {
                let left = self.get_i32(left_address)?;
                let right = self.get_i32(right_address)?;
                let remainder = left % right;

                self.set_register(destination, Register::new(remainder))?;
            }
            OperandType::I_64 => {
                let left = self.get_i64(left_address)?;
                let right = self.get_i64(right_address)?;
                let remainder = left % right;

                self.set_registers_to_i64(remainder, destination)?;
            }
            OperandType::I_128 => {
                let left = self.get_i128(left_address)?;
                let right = self.get_i128(right_address)?;
                let remainder = left % right;

                self.set_registers_to_i128(remainder, destination)?;
            }
            OperandType::U_8 => {
                let left = self.get_u8(left_address)?;
                let right = self.get_u8(right_address)?;
                let remainder = left % right;

                self.set_register(destination, Register::new(remainder))?;
            }
            OperandType::U_16 => {
                let left = self.get_u16(left_address)?;
                let right = self.get_u16(right_address)?;
                let remainder = left % right;

                self.set_register(destination, Register::new(remainder))?;
            }
            OperandType::U_32 => {
                let left = self.get_u32(left_address)?;
                let right = self.get_u32(right_address)?;
                let remainder = left % right;

                self.set_register(destination, Register::new(remainder))?;
            }
            OperandType::U_64 => {
                let left = self.get_u64(left_address)?;
                let right = self.get_u64(right_address)?;
                let remainder = left % right;

                self.set_registers_to_u64(remainder, destination)?;
            }
            OperandType::U_128 => {
                let left = self.get_u128(left_address)?;
                let right = self.get_u128(right_address)?;
                let remainder = left % right;

                self.set_registers_to_u128(remainder, destination)?;
            }
            OperandType::F_32 => {
                let left = self.get_f32(left_address)?;
                let right = self.get_f32(right_address)?;
                let remainder = left % right;

                self.set_register(destination, Register::new(remainder))?;
            }
            OperandType::F_64 => {
                let left = self.get_f64(left_address)?;
                let right = self.get_f64(right_address)?;
                let remainder = left % right;

                self.set_registers_to_f64(remainder, destination)?;
            }
            _ => return Err(VmError::UnsupportedOperandType { operand_type }),
        }

        self.frame.instruction_pointer += 1;

        Ok(())
    }

    fn exponent(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let Exponent {
            destination,
            operand_type,
            base_address,
            exponent_address,
        } = Exponent::from(instruction);

        match operand_type {
            OperandType::F_32 => {
                let base = self.get_f32(base_address)?;
                let exponent = self.get_f32(exponent_address)?;
                let result = base.powf(exponent);

                self.set_register(destination, Register::new(result.to_bits()))?;
            }
            OperandType::F_64 => {
                let base = self.get_f64(base_address)?;
                let exponent = self.get_f64(exponent_address)?;
                let result = base.powf(exponent);

                self.set_registers_to_f64(result, destination)?;
            }
            _ => return Err(VmError::UnsupportedOperandType { operand_type }),
        }

        self.frame.instruction_pointer += 1;

        Ok(())
    }

    fn get_i8(&self, Address { memory, index }: Address) -> Result<i8, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as i8),
            MemoryKind::REGISTER => {
                Ok(self.register_stack[self.frame.base_register + index as usize].as_value())
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_i16(&self, Address { memory, index }: Address) -> Result<i16, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as i16),
            MemoryKind::REGISTER => {
                Ok(self.register_stack[self.frame.base_register + index as usize].as_value())
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_i32(&self, Address { memory, index }: Address) -> Result<i32, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as i32),
            MemoryKind::CONSTANT => Ok(self.constants.get_i32(index)?),
            MemoryKind::REGISTER => {
                Ok(self.register_stack[self.frame.base_register + index as usize].as_value())
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_i64(&self, Address { memory, index }: Address) -> Result<i64, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as i64),
            MemoryKind::CONSTANT => Ok(self.constants.get_i64(index)?),
            MemoryKind::REGISTER => {
                let low_bits =
                    self.register_stack[self.frame.base_register + index as usize].as_bits() as i64;
                let high_bits = self.register_stack[self.frame.base_register + index as usize + 1]
                    .as_bits() as i64;

                Ok(high_bits << 32 | low_bits)
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_i128(&self, Address { memory, index }: Address) -> Result<i128, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as i128),
            MemoryKind::CONSTANT => Ok(self.constants.get_i128(index)?),
            MemoryKind::REGISTER => {
                let low_bits = self.register_stack[self.frame.base_register + index as usize]
                    .as_bits() as i128;
                let low_mid_bits = self.register_stack
                    [self.frame.base_register + index as usize + 1]
                    .as_bits() as i128;
                let high_mid_bits = self.register_stack
                    [self.frame.base_register + index as usize + 2]
                    .as_bits() as i128;
                let high_bits = self.register_stack[self.frame.base_register + index as usize + 3]
                    .as_bits() as i128;
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
            MemoryKind::REGISTER => {
                Ok(self.register_stack[self.frame.base_register + index as usize].as_value())
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_u16(&self, Address { memory, index }: Address) -> Result<u16, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index),
            MemoryKind::REGISTER => {
                Ok(self.register_stack[self.frame.base_register + index as usize].as_value())
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_u32(&self, Address { memory, index }: Address) -> Result<u32, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as u32),
            MemoryKind::CONSTANT => Ok(self.constants.get_u32(index)?),
            MemoryKind::REGISTER => {
                Ok(self.register_stack[self.frame.base_register + index as usize].as_value())
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_u64(&self, Address { memory, index }: Address) -> Result<u64, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as u64),
            MemoryKind::CONSTANT => Ok(self.constants.get_u64(index)?),
            MemoryKind::REGISTER => {
                let low_bits =
                    self.register_stack[self.frame.base_register + index as usize].as_bits() as u64;
                let high_bits = self.register_stack[self.frame.base_register + index as usize + 1]
                    .as_bits() as u64;

                Ok(high_bits << 32 | low_bits)
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_u128(&self, Address { memory, index }: Address) -> Result<u128, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as u128),
            MemoryKind::CONSTANT => Ok(self.constants.get_u128(index)?),
            MemoryKind::REGISTER => {
                let low_bits = self.register_stack[self.frame.base_register + index as usize]
                    .as_bits() as u128;
                let low_mid_bits = self.register_stack
                    [self.frame.base_register + index as usize + 1]
                    .as_bits() as u128;
                let high_mid_bits = self.register_stack
                    [self.frame.base_register + index as usize + 2]
                    .as_bits() as u128;
                let high_bits = self.register_stack[self.frame.base_register + index as usize + 3]
                    .as_bits() as u128;

                Ok(high_bits << 96 | high_mid_bits << 64 | low_mid_bits << 32 | low_bits)
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_f32(&self, Address { memory, index }: Address) -> Result<f32, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as f32),
            MemoryKind::CONSTANT => Ok(self.constants.get_f32(index)?),
            MemoryKind::REGISTER => {
                Ok(self.register_stack[self.frame.base_register + index as usize].as_value())
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_f64(&self, Address { memory, index }: Address) -> Result<f64, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as f64),
            MemoryKind::CONSTANT => Ok(self.constants.get_f64(index)?),
            MemoryKind::REGISTER => {
                let low_bits =
                    self.register_stack[self.frame.base_register + index as usize].as_bits() as u64;
                let high_bits = self.register_stack[self.frame.base_register + index as usize + 1]
                    .as_bits() as u64;

                Ok(f64::from_bits(high_bits << 32 | low_bits))
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn set_register(&mut self, destination: u16, register: Register) -> Result<(), VmError> {
        let index = self.frame.base_register + destination as usize;

        if index >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex { index: destination });
        }

        self.register_stack[index] = register;

        Ok(())
    }

    fn set_double_registers(
        &mut self,
        destination: u16,
        low: Register,
        high: Register,
    ) -> Result<(), VmError> {
        let first = self.frame.base_register + destination as usize;

        if first + 1 >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex { index: destination });
        }

        self.register_stack[first] = high;
        self.register_stack[first + 1] = low;

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
        let first = self.frame.base_register + destination as usize;

        if first + 3 >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex { index: destination });
        }

        self.register_stack[first] = high;
        self.register_stack[first + 1] = high_mid;
        self.register_stack[first + 2] = low_mid;
        self.register_stack[first + 3] = low;

        Ok(())
    }

    fn set_registers_to_i64(&mut self, value: i64, destination: u16) -> Result<(), VmError> {
        let low_bits = (value & 0xFFFFFFFF) as u32;
        let high_bits = ((value >> 32) & 0xFFFFFFFF) as u32;

        self.set_double_registers(
            destination,
            Register::new(low_bits),
            Register::new(high_bits),
        )
    }

    fn set_registers_to_i128(&mut self, value: i128, destination: u16) -> Result<(), VmError> {
        let low_bits = (value & 0xFFFFFFFF) as u32;
        let low_mid_bits = ((value >> 32) & 0xFFFFFFFF) as u32;
        let high_mid_bits = ((value >> 64) & 0xFFFFFFFF) as u32;
        let high_bits = ((value >> 96) & 0xFFFFFFFF) as u32;

        self.set_quad_registers(
            destination,
            Register::new(low_bits),
            Register::new(low_mid_bits),
            Register::new(high_mid_bits),
            Register::new(high_bits),
        )
    }

    fn set_registers_to_u64(&mut self, value: u64, destination: u16) -> Result<(), VmError> {
        let low_bits = (value & 0xFFFFFFFF) as u32;
        let high_bits = ((value >> 32) & 0xFFFFFFFF) as u32;

        self.set_double_registers(
            destination,
            Register::new(low_bits),
            Register::new(high_bits),
        )
    }

    fn set_registers_to_u128(&mut self, value: u128, destination: u16) -> Result<(), VmError> {
        let low_bits = (value & 0xFFFFFFFF) as u32;
        let low_mid_bits = ((value >> 32) & 0xFFFFFFFF) as u32;
        let high_mid_bits = ((value >> 64) & 0xFFFFFFFF) as u32;
        let high_bits = ((value >> 96) & 0xFFFFFFFF) as u32;

        self.set_quad_registers(
            destination,
            Register::new(low_bits),
            Register::new(low_mid_bits),
            Register::new(high_mid_bits),
            Register::new(high_bits),
        )
    }

    fn set_registers_to_f64(&mut self, value: f64, destination: u16) -> Result<(), VmError> {
        let bits = value.to_bits();
        let low_bits = (bits & 0xFFFFFFFF) as u32;
        let high_bits = (bits >> 32 & 0xFFFFFFFF) as u32;

        self.set_double_registers(
            destination,
            Register::new(low_bits),
            Register::new(high_bits),
        )
    }

    fn set_registers_to_reference(
        &mut self,
        source: u16,
        length: u16,
        destination: u16,
    ) -> Result<(), VmError> {
        let source = self.frame.base_register as u32 + source as u32;

        self.set_double_registers(
            destination,
            Register::new(source),
            Register::new(length as u32),
        )
    }

    fn copy_registers_to_registers(
        &mut self,
        operand_index: u16,
        operand_type: OperandType,
        destination: u16,
    ) -> Result<(), VmError> {
        let operand_index = self.frame.base_register + operand_index as usize;
        let count = operand_type.register_width().as_usize();
        let operand_range = operand_index..operand_index + count;
        let destination = self.frame.base_register + destination as usize;

        if operand_range.end > self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: operand_index as u16,
            });
        }

        if destination + count >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: destination as u16,
            });
        }

        self.register_stack.copy_within(operand_range, destination);

        Ok(())
    }

    fn copy_encoded_to_register(
        &mut self,
        operand_index: u16,
        operand_type: OperandType,
        destination: u16,
    ) -> Result<(), VmError> {
        let register = match operand_type {
            OperandType::BOOLEAN => Register::new((operand_index != 0) as u32),
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
            | OperandType::FUNCTION => Register::new(operand_index as u32),
            _ => return Err(VmError::UnsupportedOperandType { operand_type }),
        };
        self.register_stack[self.frame.base_register + destination as usize] = register;

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

                self.set_register(destination, Register::new(constant))?;
            }
            OperandType::I_64 => {
                let constant = self.constants.get_i64(operand_index)?;

                self.set_registers_to_i64(constant, destination)?;
            }
            OperandType::I_128 => {
                let constant = self.constants.get_i128(operand_index)?;

                self.set_registers_to_i128(constant, destination)?;
            }
            OperandType::U_32 => {
                let constant = self.constants.get_u32(operand_index)?;

                self.set_register(destination, Register::new(constant))?;
            }
            OperandType::U_64 => {
                let constant = self.constants.get_u64(operand_index)?;

                self.set_registers_to_u64(constant, destination)?;
            }
            OperandType::U_128 => {
                let constant = self.constants.get_u128(operand_index)?;

                self.set_registers_to_u128(constant, destination)?;
            }
            OperandType::F_32 => {
                let constant = self.constants.get_f32(operand_index)?;

                self.set_register(destination, Register::new(constant.to_bits()))?;
            }
            OperandType::F_64 => {
                let constant = self.constants.get_f64(operand_index)?;

                self.set_registers_to_f64(constant, destination)?;
            }
            OperandType::CHARACTER => {
                let constant = self.constants.get_character(operand_index)?;

                self.set_register(destination, Register::new(constant as u32))?;
            }
            _ => return Err(VmError::UnsupportedOperandType { operand_type }),
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CallFrame {
    pub prototype_index: usize,
    pub base_register: usize,
    pub instruction_pointer: usize,
}
