use dust_compiler::{
    constants::Constants,
    instruction::{
        Add, Address, Divide, Equal, Exponent, Get, Instruction, Jump, Less, LessEqual, MemoryKind,
        Modulo, Move, Multiply, Negate, OperandType, Operation, Reference, RegisterWidth, Set,
        Subtract, Test,
    },
    prototype::Prototype,
};
use tracing::debug;

use crate::{error::VmError, register::Register};

pub struct Call<'a> {
    frame: CallFrame,

    prototypes: &'a [Prototype],

    register_stack: &'a mut Vec<Register>,

    call_stack: &'a mut Vec<CallFrame>,

    constants: &'a Constants,
}

impl<'a> Call<'a> {
    pub fn new(
        prototypes: &'a [Prototype],
        constants: &'a Constants,
        register_stack: &'a mut Vec<Register>,
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

            debug!("IP {}: {operation}", self.frame.instruction_pointer);

            match operation {
                Operation::MOVE => self.r#move(instruction)?,
                Operation::REFERENCE => self.r#reference(instruction)?,
                Operation::GET => self.r#get(instruction)?,
                Operation::SET => self.r#set(instruction)?,
                Operation::EQUAL => self.equal(instruction)?,
                Operation::LESS => self.less(instruction)?,
                Operation::LESS_EQUAL => self.less_equal(instruction)?,
                Operation::TEST => self.test(instruction)?,
                Operation::ADD => self.add(instruction)?,
                Operation::SUBTRACT => self.subtract(instruction)?,
                Operation::MULTIPLY => self.multiply(instruction)?,
                Operation::DIVIDE => self.divide(instruction)?,
                Operation::MODULO => self.modulo(instruction)?,
                Operation::EXPONENT => self.exponent(instruction)?,
                Operation::NEGATE => self.negate(instruction)?,
                Operation::JUMP => self.jump(instruction)?,
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
                self.copy_encoded_to_register(destination, operand.index, operand_type)?;
            }
            MemoryKind::REGISTER => {
                self.copy_registers_to_registers(destination, operand.index, operand_type)?;
            }
            MemoryKind::CONSTANT => {
                self.copy_constant_to_registers(destination, operand.index, operand_type)?;
            }
            _ => {
                return Err(VmError::UnsupportedMemoryKind {
                    memory: operand.memory,
                });
            }
        }

        if jump_distance > 0 {
            if jump_forward {
                self.frame.instruction_pointer += jump_distance as usize + 1;
            } else {
                self.frame.instruction_pointer -= jump_distance as usize + 1;
            }
        } else {
            self.frame.instruction_pointer += 1;
        }

        Ok(())
    }

    fn reference(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let Reference {
            destination,
            start_register,
            end_register,
        } = Reference::from(instruction);

        let absolute_start = self.frame.base_register as u32 + start_register as u32;
        let absolute_end = self.frame.base_register as u32 + end_register as u32;

        self.set_double_registers(
            destination,
            Register::new(absolute_start),
            Register::new(absolute_end),
        )?;

        self.frame.instruction_pointer += 1;

        Ok(())
    }

    fn get(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let Get {
            destination,
            operand_type,
            base_register,
            index,
        } = Get::from(instruction);

        let absolute_base = self.frame.base_register + base_register as usize;
        let index_value = self.get_u32(index)? as usize;

        match operand_type.register_width() {
            RegisterWidth::Single => {
                let register = self.register_stack[absolute_base + index_value];

                self.set_register(destination, register)?;
            }
            RegisterWidth::Double => {
                let low = self.register_stack[absolute_base + index_value];
                let high = self.register_stack[absolute_base + index_value + 1];

                self.set_double_registers(destination, low, high)?;
            }
            RegisterWidth::Quad => {
                let low = self.register_stack[absolute_base + index_value];
                let low_mid = self.register_stack[absolute_base + index_value + 1];
                let high_mid = self.register_stack[absolute_base + index_value + 2];
                let high = self.register_stack[absolute_base + index_value + 3];

                self.set_quad_registers(destination, low, low_mid, high_mid, high)?;
            }
        };

        self.frame.instruction_pointer += 1;

        Ok(())
    }

    fn set(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let Set {
            base,
            operand_type,
            index,
            source,
        } = Set::from(instruction);

        let index_value = self.get_u32(index)?;
        let destination = base as u32 + index_value;

        match operand_type {
            OperandType::BOOLEAN => {
                let value = self.get_boolean(source)?;

                self.set_register(destination, Register::new(value))?;
            }
            OperandType::I_8 => {
                let value = self.get_i8(source)?;

                self.set_register(destination, Register::new(value))?;
            }
            OperandType::I_16 => {
                let value = self.get_i16(source)?;

                self.set_register(destination, Register::new(value))?;
            }
            OperandType::I_32 => {
                let value = self.get_i32(source)?;

                self.set_register(destination, Register::new(value))?;
            }
            OperandType::I_64 => {
                let value = self.get_i64(source)?;

                self.set_registers_to_i64(destination, value)?;
            }
            OperandType::I_128 => {
                let value = self.get_i128(source)?;

                self.set_registers_to_i128(destination, value)?;
            }
            OperandType::U_8 => {
                let value = self.get_u8(source)?;

                self.set_register(destination, Register::new(value))?;
            }
            OperandType::U_16 => {
                let value = self.get_u16(source)?;

                self.set_register(destination, Register::new(value))?;
            }
            OperandType::U_32 => {
                let value = self.get_u32(source)?;

                self.set_register(destination, Register::new(value))?;
            }
            OperandType::U_64 => {
                let value = self.get_u64(source)?;

                self.set_registers_to_u64(destination, value)?;
            }
            OperandType::U_128 => {
                let value = self.get_u128(source)?;

                self.set_registers_to_u128(destination, value)?;
            }
            OperandType::F_32 => {
                let value = self.get_f32(source)?;

                self.set_register(destination, Register::new(value.to_bits()))?;
            }
            OperandType::F_64 => {
                let value = self.get_f64(source)?;

                self.set_registers_to_f64(destination, value)?;
            }
            OperandType::CHARACTER => {
                let value = self.get_character(source)?;

                self.set_register(destination, Register::from_character(value))?;
            }
            OperandType::POINTER => {
                self.copy_registers_to_registers(destination, source.index, operand_type)?;
            }
            _ => {
                return Err(VmError::UnsupportedOperandType { operand_type });
            }
        }

        self.frame.instruction_pointer += 1;

        Ok(())
    }

    fn equal(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let Equal {
            comparator,
            operand_type,
            left_address,
            right_address,
        } = Equal::from(instruction);

        let equals = match operand_type {
            OperandType::BOOLEAN => {
                let left = self.get_boolean(left_address)?;
                let right = self.get_boolean(right_address)?;

                left == right
            }
            OperandType::I_8 => {
                let left = self.get_i8(left_address)?;
                let right = self.get_i8(right_address)?;

                left == right
            }
            OperandType::I_16 => {
                let left = self.get_i16(left_address)?;
                let right = self.get_i16(right_address)?;

                left == right
            }
            OperandType::I_32 => {
                let left = self.get_i32(left_address)?;
                let right = self.get_i32(right_address)?;

                left == right
            }
            OperandType::I_64 => {
                let left = self.get_i64(left_address)?;
                let right = self.get_i64(right_address)?;

                left == right
            }
            OperandType::I_128 => {
                let left = self.get_i128(left_address)?;
                let right = self.get_i128(right_address)?;

                left == right
            }
            OperandType::U_8 => {
                let left = self.get_u8(left_address)?;
                let right = self.get_u8(right_address)?;

                left == right
            }
            OperandType::U_16 => {
                let left = self.get_u16(left_address)?;
                let right = self.get_u16(right_address)?;

                left == right
            }
            OperandType::U_32 => {
                let left = self.get_u32(left_address)?;
                let right = self.get_u32(right_address)?;

                left == right
            }
            OperandType::U_64 => {
                let left = self.get_u64(left_address)?;
                let right = self.get_u64(right_address)?;

                left == right
            }
            OperandType::U_128 => {
                let left = self.get_u128(left_address)?;
                let right = self.get_u128(right_address)?;

                left == right
            }
            OperandType::F_32 => {
                let left = self.get_f32(left_address)?;
                let right = self.get_f32(right_address)?;

                left == right
            }
            OperandType::F_64 => {
                let left = self.get_f64(left_address)?;
                let right = self.get_f64(right_address)?;

                left == right
            }
            OperandType::CHARACTER => {
                let left = self.get_character(left_address)?;
                let right = self.get_character(right_address)?;

                left == right
            }
            OperandType::POINTER => {
                let (left_start, left_end) = self.get_reference(left_address.index)?;
                let (right_start, right_end) = self.get_reference(right_address.index)?;

                left_start == right_start && left_end == right_end
            }
            _ => return Err(VmError::UnsupportedOperandType { operand_type }),
        };

        if equals == comparator {
            self.frame.instruction_pointer += 2;
        } else {
            self.frame.instruction_pointer += 1;
        }

        Ok(())
    }

    fn less(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let Less {
            comparator,
            operand_type,
            left_address,
            right_address,
        } = Less::from(instruction);

        let less_than = match operand_type {
            OperandType::I_8 => {
                let left = self.get_i8(left_address)?;
                let right = self.get_i8(right_address)?;

                left < right
            }
            OperandType::I_16 => {
                let left = self.get_i16(left_address)?;
                let right = self.get_i16(right_address)?;

                left < right
            }
            OperandType::I_32 => {
                let left = self.get_i32(left_address)?;
                let right = self.get_i32(right_address)?;

                left < right
            }
            OperandType::I_64 => {
                let left = self.get_i64(left_address)?;
                let right = self.get_i64(right_address)?;

                left < right
            }
            OperandType::I_128 => {
                let left = self.get_i128(left_address)?;
                let right = self.get_i128(right_address)?;

                left < right
            }
            OperandType::U_8 => {
                let left = self.get_u8(left_address)?;
                let right = self.get_u8(right_address)?;

                left < right
            }
            OperandType::U_16 => {
                let left = self.get_u16(left_address)?;
                let right = self.get_u16(right_address)?;

                left < right
            }
            OperandType::U_32 => {
                let left = self.get_u32(left_address)?;
                let right = self.get_u32(right_address)?;

                left < right
            }
            OperandType::U_64 => {
                let left = self.get_u64(left_address)?;
                let right = self.get_u64(right_address)?;

                left < right
            }
            OperandType::U_128 => {
                let left = self.get_u128(left_address)?;
                let right = self.get_u128(right_address)?;

                left < right
            }
            OperandType::F_32 => {
                let left = self.get_f32(left_address)?;
                let right = self.get_f32(right_address)?;

                left < right
            }
            OperandType::F_64 => {
                let left = self.get_f64(left_address)?;
                let right = self.get_f64(right_address)?;

                left < right
            }
            OperandType::CHARACTER => {
                let left = self.get_character(left_address)?;
                let right = self.get_character(right_address)?;

                left < right
            }
            OperandType::POINTER => {
                let (left_start, left_end) = self.get_reference(left_address.index)?;
                let (right_start, right_end) = self.get_reference(right_address.index)?;

                left_start < right_start || (left_start == right_start && left_end < right_end)
            }
            _ => return Err(VmError::UnsupportedOperandType { operand_type }),
        };

        if less_than == comparator {
            self.frame.instruction_pointer += 2;
        } else {
            self.frame.instruction_pointer += 1;
        }

        Ok(())
    }

    fn less_equal(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let LessEqual {
            comparator,
            operand_type,
            left_address,
            right_address,
        } = LessEqual::from(instruction);

        let less_than_or_equal = match operand_type {
            OperandType::I_8 => {
                let left = self.get_i8(left_address)?;
                let right = self.get_i8(right_address)?;

                left <= right
            }
            OperandType::I_16 => {
                let left = self.get_i16(left_address)?;
                let right = self.get_i16(right_address)?;

                left <= right
            }
            OperandType::I_32 => {
                let left = self.get_i32(left_address)?;
                let right = self.get_i32(right_address)?;

                left <= right
            }
            OperandType::I_64 => {
                let left = self.get_i64(left_address)?;
                let right = self.get_i64(right_address)?;

                left <= right
            }
            OperandType::I_128 => {
                let left = self.get_i128(left_address)?;
                let right = self.get_i128(right_address)?;

                left <= right
            }
            OperandType::U_8 => {
                let left = self.get_u8(left_address)?;
                let right = self.get_u8(right_address)?;

                left <= right
            }
            OperandType::U_16 => {
                let left = self.get_u16(left_address)?;
                let right = self.get_u16(right_address)?;

                left <= right
            }
            OperandType::U_32 => {
                let left = self.get_u32(left_address)?;
                let right = self.get_u32(right_address)?;

                left <= right
            }
            OperandType::U_64 => {
                let left = self.get_u64(left_address)?;
                let right = self.get_u64(right_address)?;

                left <= right
            }
            OperandType::U_128 => {
                let left = self.get_u128(left_address)?;
                let right = self.get_u128(right_address)?;

                left <= right
            }
            OperandType::F_32 => {
                let left = self.get_f32(left_address)?;
                let right = self.get_f32(right_address)?;

                left <= right
            }
            OperandType::F_64 => {
                let left = self.get_f64(left_address)?;
                let right = self.get_f64(right_address)?;

                left <= right
            }
            OperandType::CHARACTER => {
                let left = self.get_character(left_address)?;
                let right = self.get_character(right_address)?;

                left <= right
            }
            OperandType::POINTER => {
                let (left_start, left_end) = self.get_reference(left_address.index)?;
                let (right_start, right_end) = self.get_reference(right_address.index)?;

                left_start < right_start || (left_start == right_start && left_end <= right_end)
            }
            _ => return Err(VmError::UnsupportedOperandType { operand_type }),
        };

        if less_than_or_equal == comparator {
            self.frame.instruction_pointer += 2;
        } else {
            self.frame.instruction_pointer += 1;
        }

        Ok(())
    }

    fn test(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let Test {
            comparator,
            operand,
            jump_distance,
        } = Test::from(instruction);

        let value = self.get_boolean(operand)?;

        if value == comparator {
            self.frame.instruction_pointer += jump_distance as usize + 1;
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

                self.set_registers_to_i64(destination, sum)?;
            }
            OperandType::I_128 => {
                let left = self.get_i128(left_address)?;
                let right = self.get_i128(right_address)?;
                let sum = left + right;

                self.set_registers_to_i128(destination, sum)?;
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

                self.set_registers_to_u64(destination, sum)?;
            }
            OperandType::U_128 => {
                let left = self.get_u128(left_address)?;
                let right = self.get_u128(right_address)?;
                let sum = left + right;

                self.set_registers_to_u128(destination, sum)?;
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

                self.set_registers_to_f64(destination, sum)?;
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

                self.set_registers_to_i64(destination, difference)?;
            }
            OperandType::I_128 => {
                let left = self.get_i128(left_address)?;
                let right = self.get_i128(right_address)?;
                let difference = left - right;

                self.set_registers_to_i128(destination, difference)?;
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

                self.set_registers_to_u64(destination, difference)?;
            }
            OperandType::U_128 => {
                let left = self.get_u128(left_address)?;
                let right = self.get_u128(right_address)?;
                let difference = left - right;

                self.set_registers_to_u128(destination, difference)?;
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

                self.set_registers_to_f64(destination, difference)?;
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

                self.set_registers_to_i64(destination, product)?;
            }
            OperandType::I_128 => {
                let left = self.get_i128(left_address)?;
                let right = self.get_i128(right_address)?;
                let product = left * right;

                self.set_registers_to_i128(destination, product)?;
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

                self.set_registers_to_u64(destination, product)?;
            }
            OperandType::U_128 => {
                let left = self.get_u128(left_address)?;
                let right = self.get_u128(right_address)?;
                let product = left * right;

                self.set_registers_to_u128(destination, product)?;
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

                self.set_registers_to_f64(destination, product)?;
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

                self.set_registers_to_i64(destination, quotient)?;
            }
            OperandType::I_128 => {
                let left = self.get_i128(left_address)?;
                let right = self.get_i128(right_address)?;
                let quotient = left / right;

                self.set_registers_to_i128(destination, quotient)?;
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

                self.set_registers_to_u64(destination, quotient)?;
            }
            OperandType::U_128 => {
                let left = self.get_u128(left_address)?;
                let right = self.get_u128(right_address)?;
                let quotient = left / right;

                self.set_registers_to_u128(destination, quotient)?;
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

                self.set_registers_to_f64(destination, quotient)?;
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

                self.set_registers_to_i64(destination, remainder)?;
            }
            OperandType::I_128 => {
                let left = self.get_i128(left_address)?;
                let right = self.get_i128(right_address)?;
                let remainder = left % right;

                self.set_registers_to_i128(destination, remainder)?;
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

                self.set_registers_to_u64(destination, remainder)?;
            }
            OperandType::U_128 => {
                let left = self.get_u128(left_address)?;
                let right = self.get_u128(right_address)?;
                let remainder = left % right;

                self.set_registers_to_u128(destination, remainder)?;
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

                self.set_registers_to_f64(destination, remainder)?;
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

                self.set_registers_to_f64(destination, result)?;
            }
            _ => return Err(VmError::UnsupportedOperandType { operand_type }),
        }

        self.frame.instruction_pointer += 1;

        Ok(())
    }

    fn negate(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let Negate {
            destination,
            operand_type,
            operand,
        } = Negate::from(instruction);

        match operand_type {
            OperandType::BOOLEAN => {
                let value = self.get_boolean(operand)?;
                let negated_value = !value;

                self.set_register(destination, Register::new(negated_value as u32))?;
            }
            OperandType::I_8 => {
                let value = self.get_i8(operand)?;
                let negated_value = -value;

                self.set_register(destination, Register::new(negated_value))?;
            }
            OperandType::I_16 => {
                let value = self.get_i16(operand)?;
                let negated_value = -value;

                self.set_register(destination, Register::new(negated_value))?;
            }
            OperandType::I_32 => {
                let value = self.get_i32(operand)?;
                let negated_value = -value;

                self.set_register(destination, Register::new(negated_value))?;
            }
            OperandType::I_64 => {
                let value = self.get_i64(operand)?;
                let negated_value = -value;

                self.set_registers_to_i64(destination, negated_value)?;
            }
            OperandType::I_128 => {
                let value = self.get_i128(operand)?;
                let negated_value = -value;

                self.set_registers_to_i128(destination, negated_value)?;
            }
            OperandType::F_32 => {
                let value = self.get_f32(operand)?;
                let negated_value = -value;

                self.set_register(destination, Register::new(negated_value.to_bits()))?;
            }
            OperandType::F_64 => {
                let value = self.get_f64(operand)?;
                let negated_value = -value;

                self.set_registers_to_f64(destination, negated_value)?;
            }
            _ => return Err(VmError::UnsupportedOperandType { operand_type }),
        }

        self.frame.instruction_pointer += 1;

        Ok(())
    }

    fn jump(&mut self, instruction: Instruction) -> Result<(), VmError> {
        let Jump {
            offset,
            is_positive,
            drop_register_start,
            drop_list_end,
        } = Jump::from(instruction);

        if is_positive {
            self.frame.instruction_pointer += offset as usize + 1;
        } else {
            self.frame.instruction_pointer -= offset as usize + 1;
        }

        Ok(())
    }

    fn get_boolean(&self, Address { memory, index }: Address) -> Result<bool, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index != 0),
            MemoryKind::REGISTER => {
                let value =
                    self.register_stack[self.frame.base_register + index as usize].as_value();

                Ok(value)
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_i8(&self, Address { memory, index }: Address) -> Result<i8, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as i8),
            MemoryKind::REGISTER => {
                let value =
                    self.register_stack[self.frame.base_register + index as usize].as_value();

                Ok(value)
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_i16(&self, Address { memory, index }: Address) -> Result<i16, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as i16),
            MemoryKind::REGISTER => {
                let value =
                    self.register_stack[self.frame.base_register + index as usize].as_value();

                Ok(value)
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_i32(&self, Address { memory, index }: Address) -> Result<i32, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(index as i32),
            MemoryKind::CONSTANT => Ok(self.constants.get_i32(index)?),
            MemoryKind::REGISTER => {
                let value =
                    self.register_stack[self.frame.base_register + index as usize].as_value();

                Ok(value)
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

    fn get_character(&self, Address { memory, index }: Address) -> Result<char, VmError> {
        match memory {
            MemoryKind::ENCODED => Ok(char::from_u32(index as u32).ok_or(
                VmError::InvalidCharacter {
                    value: index as u32,
                },
            )?),
            MemoryKind::CONSTANT => Ok(self.constants.get_character(index)?),
            MemoryKind::REGISTER => {
                let value = self.register_stack[self.frame.base_register + index as usize]
                    .as_character()?;

                Ok(value)
            }
            _ => Err(VmError::UnsupportedMemoryKind { memory }),
        }
    }

    fn get_reference(&self, index: u16) -> Result<(Register, Register), VmError> {
        let low_index = self.frame.base_register + index as usize;
        let high_index = self.frame.base_register + index as usize + 1;

        if high_index >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: index as usize,
            });
        }

        let start = self.register_stack[low_index];
        let end = self.register_stack[high_index];

        Ok((start, end))
    }

    fn set_register(
        &mut self,
        destination: impl Destination,
        register: Register,
    ) -> Result<(), VmError> {
        let index = self.frame.base_register + destination.as_usize();

        if index >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: destination.as_usize(),
            });
        }

        self.register_stack[index] = register;

        Ok(())
    }

    fn set_double_registers(
        &mut self,
        destination: impl Destination,
        low: Register,
        high: Register,
    ) -> Result<(), VmError> {
        let first = self.frame.base_register + destination.as_usize();

        if first + 1 >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: destination.as_usize(),
            });
        }

        self.register_stack[first] = high;
        self.register_stack[first + 1] = low;

        Ok(())
    }

    fn set_quad_registers(
        &mut self,
        destination: impl Destination,
        low: Register,
        low_mid: Register,
        high_mid: Register,
        high: Register,
    ) -> Result<(), VmError> {
        let first = self.frame.base_register + destination.as_usize();

        if first + 3 >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: destination.as_usize(),
            });
        }

        self.register_stack[first] = high;
        self.register_stack[first + 1] = high_mid;
        self.register_stack[first + 2] = low_mid;
        self.register_stack[first + 3] = low;

        Ok(())
    }

    fn set_registers_to_i64(
        &mut self,
        destination: impl Destination,
        value: i64,
    ) -> Result<(), VmError> {
        let low_bits = (value & 0xFFFFFFFF) as u32;
        let high_bits = ((value >> 32) & 0xFFFFFFFF) as u32;

        self.set_double_registers(
            destination,
            Register::new(low_bits),
            Register::new(high_bits),
        )
    }

    fn set_registers_to_i128(
        &mut self,
        destination: impl Destination,
        value: i128,
    ) -> Result<(), VmError> {
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

    fn set_registers_to_u64(
        &mut self,
        destination: impl Destination,
        value: u64,
    ) -> Result<(), VmError> {
        let low_bits = (value & 0xFFFFFFFF) as u32;
        let high_bits = ((value >> 32) & 0xFFFFFFFF) as u32;

        self.set_double_registers(
            destination,
            Register::new(low_bits),
            Register::new(high_bits),
        )
    }

    fn set_registers_to_u128(
        &mut self,
        destination: impl Destination,
        value: u128,
    ) -> Result<(), VmError> {
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

    fn set_registers_to_f64(
        &mut self,
        destination: impl Destination,
        value: f64,
    ) -> Result<(), VmError> {
        let bits = value.to_bits();
        let low_bits = (bits & 0xFFFFFFFF) as u32;
        let high_bits = (bits >> 32 & 0xFFFFFFFF) as u32;

        self.set_double_registers(
            destination,
            Register::new(low_bits),
            Register::new(high_bits),
        )
    }

    fn copy_registers_to_registers(
        &mut self,
        destination: impl Destination,
        operand_index: u16,
        operand_type: OperandType,
    ) -> Result<(), VmError> {
        let operand_index = self.frame.base_register + operand_index as usize;
        let count = operand_type.register_width().as_usize();
        let operand_range = operand_index..operand_index + count;
        let destination = self.frame.base_register + destination.as_usize();

        if operand_range.end > self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: operand_index,
            });
        }

        if destination + count >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex { index: destination });
        }

        self.register_stack.copy_within(operand_range, destination);

        Ok(())
    }

    fn copy_encoded_to_register(
        &mut self,
        destination: u16,
        operand_index: u16,
        operand_type: OperandType,
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
        destination: u16,
        operand_index: u16,
        operand_type: OperandType,
    ) -> Result<(), VmError> {
        match operand_type {
            OperandType::I_32 => {
                let constant = self.constants.get_i32(operand_index)?;

                self.set_register(destination, Register::new(constant))?;
            }
            OperandType::I_64 => {
                let constant = self.constants.get_i64(operand_index)?;

                self.set_registers_to_i64(destination, constant)?;
            }
            OperandType::I_128 => {
                let constant = self.constants.get_i128(operand_index)?;

                self.set_registers_to_i128(destination, constant)?;
            }
            OperandType::U_32 => {
                let constant = self.constants.get_u32(operand_index)?;

                self.set_register(destination, Register::new(constant))?;
            }
            OperandType::U_64 => {
                let constant = self.constants.get_u64(operand_index)?;

                self.set_registers_to_u64(destination, constant)?;
            }
            OperandType::U_128 => {
                let constant = self.constants.get_u128(operand_index)?;

                self.set_registers_to_u128(destination, constant)?;
            }
            OperandType::F_32 => {
                let constant = self.constants.get_f32(operand_index)?;

                self.set_register(destination, Register::new(constant.to_bits()))?;
            }
            OperandType::F_64 => {
                let constant = self.constants.get_f64(operand_index)?;

                self.set_registers_to_f64(destination, constant)?;
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

trait Destination: Copy {
    fn as_usize(self) -> usize;
}

impl Destination for u16 {
    fn as_usize(self) -> usize {
        self as usize
    }
}

impl Destination for u32 {
    fn as_usize(self) -> usize {
        self as usize
    }
}
