use dust_compiler::{
    constants::Constants,
    instruction::{Instruction, dispatch_keys},
    prototype::Prototype,
};
use tracing::trace;

use crate::{
    error::VmError,
    register::{Register, RegisterValue},
};

pub struct Call<'a> {
    prototypes: &'a [Prototype],

    register_stack: &'a mut Vec<Register>,

    call_stack: &'a mut Vec<CallFrame>,

    constants: &'a Constants,

    instructions: &'a [Instruction],

    base_register: usize,

    instruction_pointer: usize,
}

impl<'a> Call<'a> {
    pub fn new(
        prototypes: &'a [Prototype],
        constants: &'a Constants,
        register_stack: &'a mut Vec<Register>,
        call_stack: &'a mut Vec<CallFrame>,
        main_prototype_index: u32,
    ) -> Result<Self, VmError> {
        let main_prototype = prototypes.get(main_prototype_index as usize).ok_or(
            VmError::InvalidPrototypeIndex {
                index: main_prototype_index,
            },
        )?;
        let starting_call_frame = CallFrame {
            prototype_index: main_prototype_index,
            base_register: 0,
            register_count: main_prototype.register_count,
            return_register_count: main_prototype.return_types.len() as u16,
            instruction_pointer: 0,
        };

        call_stack.push(starting_call_frame);

        Ok(Self {
            prototypes,
            register_stack,
            constants,
            call_stack,
            base_register: 0,
            instruction_pointer: 0,
            instructions: &main_prototype.instructions,
        })
    }

    pub fn run(mut self) -> Result<Option<CallFrame>, VmError> {
        while !self.call_stack.is_empty() {
            let instruction = self.instructions.get(self.instruction_pointer).ok_or(
                VmError::InvalidInstructionPointer {
                    instruction_pointer: self.instruction_pointer,
                },
            )?;
            let a_field = instruction.a_field() as usize;
            let b_field = instruction.b_field();
            let c_field = instruction.c_field();

            #[cfg(debug_assertions)]
            trace!("IP: {} | {instruction}", self.instruction_pointer);

            use dispatch_keys::*;

            match instruction.dispatch_key() {
                MOVE_BOOLEAN_REGISTER | MOVE_I32_REGISTER => {
                    self.copy_register_to_register(a_field, b_field as usize);
                    self.jump_forward(c_field as usize);
                }
                MOVE_BOOLEAN_ENCODED => {
                    self.set_register(a_field, bool::decode(b_field)?)?;
                    self.jump_forward(c_field as usize);
                }
                MOVE_I32_REGISTER_BACKWARD => {
                    self.copy_register_to_register(a_field, b_field as usize);
                    self.jump_backward(c_field as usize);
                }
                MOVE_I32_ENCODED => {
                    self.set_register(a_field, i32::decode(b_field)?)?;
                    self.jump_forward(c_field as usize);
                }
                MOVE_FUNCTION_ENCODED => {
                    self.set_register(a_field, b_field as u32)?;
                    self.jump_forward(c_field as usize);
                }
                GET_BOOLEAN_REGISTER => {
                    let operand_offset = self.get_register(c_field as usize)?.as_bits() as usize;

                    self.copy_register_to_register(a_field, b_field as usize + operand_offset);
                    self.go_forward();
                }
                SET_BOOLEAN_REGISTER_ENCODED => {
                    let destination_offset =
                        self.get_register(b_field as usize)?.as_value::<u32>() as usize;

                    self.set_register(a_field + destination_offset, bool::decode(c_field)?)?;
                    self.go_forward();
                }
                LESS_I32_REGISTER_ENCODED => {
                    let left_i32 = self.get_register(b_field as usize)?.as_value::<i32>();
                    let right_i32 = i32::decode(c_field)?;
                    let should_skip = self.less_with_comparator(left_i32, right_i32, a_field);

                    self.jump_forward(should_skip as usize);
                }
                LESS_EQUAL_REGISTER_ENCODED => {
                    let left_i32 = self.get_register(b_field as usize)?.as_value::<i32>();
                    let right_i32 = i32::decode(c_field)?;
                    let should_skip = self.less_equal_with_comparator(left_i32, right_i32, a_field);

                    self.jump_forward(should_skip as usize);
                }
                TEST_REGISTER => {
                    let value = self.get_register(b_field as usize)?.as_value::<bool>();
                    let should_skip = self.test_with_comparator(value, a_field);

                    self.jump_forward(should_skip as usize);
                }
                TEST_ENCODED => {
                    let value = bool::decode(b_field)?;
                    let should_skip = self.test_with_comparator(value, a_field);

                    self.jump_forward(should_skip as usize);
                }
                ADD_I32_REGISER_REGISTER => {
                    let sum = self.add_registers::<i32>(b_field as usize, c_field as usize)?;

                    self.set_register(a_field, sum)?;
                    self.go_forward();
                }
                ADD_I32_REGISTER_ENCODED => {
                    let sum = self.add_register_to_encoded::<i32>(b_field as usize, c_field)?;

                    self.set_register(a_field, sum)?;
                    self.go_forward();
                }
                SUBTRACT_I32_REGISTER_ENCODED => {
                    let difference =
                        self.subtract_encoded_from_register::<i32>(b_field as usize, c_field)?;

                    self.set_register(a_field, difference)?;
                    self.go_forward();
                }
                MULTIPLY_I32_REGISTER_REGISTER => {
                    let product =
                        self.multiply_registers::<i32>(b_field as usize, c_field as usize)?;

                    self.set_register(a_field, product)?;
                    self.go_forward();
                }
                NEGATE_BOOLEAN_REGISTER => {
                    let negated = self.negate_register::<bool>(b_field as usize)?;

                    self.set_register(a_field, negated)?;
                    self.go_forward();
                }
                CALL_REGISTER => {
                    let next_prototype_index =
                        self.get_register(b_field as usize)?.as_bits() as usize;

                    self.call(next_prototype_index, c_field as usize)?;
                }
                CALL_ENCODED => {
                    self.call(b_field as usize, c_field as usize)?;
                }
                JUMP_FORWARD => {
                    self.jump_forward(a_field);
                }
                JUMP_BACKWARD => {
                    self.jump_backward(a_field);
                }
                RETURN => {
                    if self.call_stack.len() == 1 {
                        break;
                    }

                    self.call_stack.pop();

                    let next_frame = *self.call_stack.last().ok_or(VmError::CallStackUnderflow)?;
                    let next_prototype = self
                        .prototypes
                        .get(next_frame.prototype_index as usize)
                        .ok_or(VmError::InvalidPrototypeIndex {
                        index: next_frame.prototype_index,
                    })?;

                    self.instructions = &next_prototype.instructions;
                    self.instruction_pointer = next_frame.instruction_pointer as usize;
                    self.base_register = next_frame.base_register as usize;
                }
                _ => {
                    return Err(VmError::InvalidInstructionDispatch {
                        instruction: *instruction,
                    });
                }
            }
        }

        Ok(self.call_stack.pop())
    }

    fn equal_with_comparator<T: PartialEq>(
        &mut self,
        left: T,
        right: T,
        comparator: usize,
    ) -> bool {
        (left == right) == (comparator != 0)
    }

    fn less_with_comparator<T: Ord>(&mut self, left: T, right: T, comparator: usize) -> bool {
        (left < right) == (comparator != 0)
    }

    fn less_equal_with_comparator<T: Ord>(&mut self, left: T, right: T, comparator: usize) -> bool {
        (left <= right) == (comparator != 0)
    }

    fn test_with_comparator(&mut self, value: bool, comparator: usize) -> bool {
        !(value && (comparator != 0))
    }

    fn add_registers<T: RegisterValue + Numeric>(
        &mut self,
        left: usize,
        right: usize,
    ) -> Result<T, VmError> {
        let left_value = self.get_register(left)?.as_value::<T>();
        let right_value = self.get_register(right)?.as_value::<T>();

        Ok(left_value.sum(right_value))
    }

    fn add_register_to_encoded<T: RegisterValue + Encoded + Numeric>(
        &mut self,
        left: usize,
        right: u64,
    ) -> Result<T, VmError> {
        let left_value = self.get_register(left)?.as_value::<T>();
        let right_value = T::decode(right)?;

        Ok(left_value.sum(right_value))
    }

    fn subtract_registers<T: RegisterValue + Numeric>(
        &mut self,
        left: usize,
        right: usize,
    ) -> Result<T, VmError> {
        let left_value = self.get_register(left)?.as_value::<T>();
        let right_value = self.get_register(right)?.as_value::<T>();

        Ok(left_value.subtract(right_value))
    }

    fn subtract_encoded_from_register<T: RegisterValue + Encoded + Numeric>(
        &mut self,
        left: usize,
        right: u64,
    ) -> Result<T, VmError> {
        let left_value = self.get_register(left)?.as_value::<T>();
        let right_value = T::decode(right)?;

        Ok(left_value.subtract(right_value))
    }

    fn multiply_registers<T: RegisterValue + Numeric>(
        &mut self,
        left: usize,
        right: usize,
    ) -> Result<T, VmError> {
        let left_value = self.get_register(left)?.as_value::<T>();
        let right_value = self.get_register(right)?.as_value::<T>();

        Ok(left_value.multiply(right_value))
    }

    fn negate_register<T: RegisterValue + Negate>(&mut self, index: usize) -> Result<T, VmError> {
        let value = self.get_register(index)?.as_value::<T>();

        Ok(value.negate())
    }

    fn call(&mut self, prototype_index: usize, register_offset: usize) -> Result<(), VmError> {
        self.call_stack
            .last_mut()
            .ok_or(VmError::CallStackUnderflow)?
            .instruction_pointer = self.instruction_pointer as u32 + 1;

        let next_prototype =
            self.prototypes
                .get(prototype_index)
                .ok_or(VmError::InvalidPrototypeIndex {
                    index: prototype_index as u32,
                })?;
        let next_frame = CallFrame {
            prototype_index: prototype_index as u32,
            base_register: (self.base_register + register_offset) as u32,
            register_count: next_prototype.register_count,
            return_register_count: next_prototype.return_types.len() as u16,
            instruction_pointer: 0,
        };

        self.call_stack.push(next_frame);

        self.base_register = next_frame.base_register as usize;
        self.instructions = &next_prototype.instructions;
        self.instruction_pointer = 0;

        Ok(())
    }

    fn go_forward(&mut self) {
        self.instruction_pointer += 1;
    }

    fn jump_forward(&mut self, distance: usize) {
        self.instruction_pointer += distance + 1;
    }

    fn jump_backward(&mut self, distance: usize) {
        self.instruction_pointer -= distance + 1;
    }

    fn copy_register_to_register(&mut self, destination: usize, operand: usize) {
        let destination = self.base_register + destination;
        let operand = self.base_register + operand;

        self.register_stack[destination] = self.register_stack[operand];
    }

    fn get_register(&self, index: usize) -> Result<&Register, VmError> {
        let absolute_index = self.base_register + index;

        if absolute_index >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: absolute_index,
            });
        }

        Ok(&self.register_stack[absolute_index])
    }

    fn get_double_registers(&self, index: usize) -> Result<(&Register, &Register), VmError> {
        let low_absolute_index = self.base_register + index;
        let high_absolute_index = low_absolute_index + 1;

        if high_absolute_index >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: high_absolute_index,
            });
        }

        Ok((
            &self.register_stack[low_absolute_index],
            &self.register_stack[high_absolute_index],
        ))
    }

    fn get_quad_registers(
        &self,
        index: usize,
    ) -> Result<(&Register, &Register, &Register, &Register), VmError> {
        let low_absolute_index = self.base_register + index;
        let low_mid_absolute_index = low_absolute_index + 1;
        let high_mid_absolute_index = low_absolute_index + 2;
        let high_absolute_index = low_absolute_index + 3;

        if high_absolute_index >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: high_absolute_index,
            });
        }

        Ok((
            &self.register_stack[low_absolute_index],
            &self.register_stack[low_mid_absolute_index],
            &self.register_stack[high_mid_absolute_index],
            &self.register_stack[high_absolute_index],
        ))
    }

    fn set_register(&mut self, index: usize, value: impl RegisterValue) -> Result<(), VmError> {
        let absolute_index = self.base_register + index;

        if absolute_index >= self.register_stack.len() {
            let new_size = self.register_stack.len() * 2;

            self.register_stack.resize(new_size, Register::default());
        }

        self.register_stack[absolute_index] = Register::new(value);

        Ok(())
    }

    fn set_i64_to_registers(&mut self, index: usize, value: i64) -> Result<(), VmError> {
        let low_absolute_index = self.base_register + index;
        let high_absolute_index = low_absolute_index + 1;

        if high_absolute_index >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: high_absolute_index,
            });
        }

        let low = (value & 0xFFFFFFFF) as u32;
        let high = ((value >> 32) & 0xFFFFFFFF) as u32;

        self.register_stack[low_absolute_index] = Register::new(low);
        self.register_stack[high_absolute_index] = Register::new(high);

        Ok(())
    }

    fn set_i128_to_registers(&mut self, index: usize, value: i128) -> Result<(), VmError> {
        let low_absolute_index = self.base_register + index;
        let low_mid_absolute_index = low_absolute_index + 1;
        let high_mid_absolute_index = low_absolute_index + 2;
        let high_absolute_index = low_absolute_index + 3;

        if high_absolute_index >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: high_absolute_index,
            });
        }

        let low = (value & 0xFFFFFFFF) as u32;
        let low_mid = ((value >> 32) & 0xFFFFFFFF) as u32;
        let high_mid = ((value >> 64) & 0xFFFFFFFF) as u32;
        let high = ((value >> 96) & 0xFFFFFFFF) as u32;

        self.register_stack[low_absolute_index] = Register::new(low);
        self.register_stack[low_mid_absolute_index] = Register::new(low_mid);
        self.register_stack[high_mid_absolute_index] = Register::new(high_mid);
        self.register_stack[high_absolute_index] = Register::new(high);

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CallFrame {
    pub prototype_index: u32,
    pub base_register: u32,
    pub register_count: u16,
    pub return_register_count: u16,
    pub instruction_pointer: u32,
}

trait Encoded: Sized {
    fn decode(index: u64) -> Result<Self, VmError>;
}

impl Encoded for bool {
    fn decode(index: u64) -> Result<Self, VmError> {
        Ok(index != 0)
    }
}

impl Encoded for i32 {
    fn decode(index: u64) -> Result<Self, VmError> {
        Ok(index as i32)
    }
}

trait Numeric {
    fn sum(self, other: Self) -> Self;
    fn subtract(self, other: Self) -> Self;
    fn multiply(self, other: Self) -> Self;
}

impl Numeric for i32 {
    fn sum(self, other: Self) -> Self {
        self.saturating_add(other)
    }

    fn subtract(self, other: Self) -> Self {
        self.saturating_sub(other)
    }

    fn multiply(self, other: Self) -> Self {
        self.saturating_mul(other)
    }
}

trait Negate {
    fn negate(self) -> Self;
}

impl Negate for bool {
    fn negate(self) -> Self {
        !self
    }
}
