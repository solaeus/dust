use dust_compiler::{constants::Constants, instruction::dispatch_keys, prototype::Prototype};

use crate::{
    error::VmError,
    register::{Register, RegisterValue},
};

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

    pub fn run(mut self) -> Result<Option<CallFrame>, VmError> {
        let prototype = self
            .prototypes
            .get(self.frame.prototype_index as usize)
            .ok_or(VmError::InvalidPrototypeIndex {
                index: self.frame.prototype_index,
            })?;
        let mut instruction_pointer = self.frame.instruction_pointer as usize;

        while !self.call_stack.is_empty() {
            let instruction = *prototype.instructions.get(instruction_pointer).ok_or(
                VmError::InvalidInstructionPointer {
                    instruction_pointer,
                },
            )?;

            let a_field = instruction.a_field_as_usize();
            let b_field = instruction.b_field_as_u64();
            let c_field = instruction.c_field_as_u64();

            use dispatch_keys::*;

            match instruction.dispatch_key() {
                MOVE_BOOLEAN_REGISTER => {
                    let boolean = self.get_register(b_field as usize)?.as_value::<bool>();

                    self.set_register(a_field, boolean)?;

                    instruction_pointer += c_field as usize + 1;
                }
                MOVE_BOOLEAN_ENCODED => {
                    let boolean = bool::decode(b_field)?;

                    self.set_register(a_field, boolean)?;

                    instruction_pointer += c_field as usize + 1;
                }
                MOVE_I32_REGISTER => {
                    let i32 = self.get_register(b_field as usize)?.as_value::<i32>();

                    self.set_register(a_field, i32)?;

                    instruction_pointer += c_field as usize + 1;
                }
                MOVE_I32_REGISTER_BACKWARD => {
                    let i32 = self.get_register(b_field as usize)?.as_value::<i32>();

                    self.set_register(a_field, i32)?;

                    instruction_pointer -= c_field as usize + 1;
                }
                MOVE_I32_ENCODED => {
                    let i32 = i32::decode(b_field)?;

                    self.set_register(a_field, i32)?;

                    instruction_pointer += c_field as usize + 1;
                }
                GET_BOOLEAN_REGISTER => {
                    let base_register = b_field as usize;
                    let offset = self.get_register(c_field as usize)?.as_bits() as usize;
                    let source_start = base_register + offset;
                    let source_end = source_start + 1;

                    self.register_stack
                        .copy_within(source_start..source_end, a_field);

                    instruction_pointer += 1;
                }
                SET_BOOLEAN_REGISTER_ENCODED => {
                    let left_index =
                        self.get_register(b_field as usize)?.as_value::<u32>() as usize;
                    let destination = a_field + left_index;
                    let value = bool::decode(c_field)?;

                    self.set_register(destination, value)?;

                    instruction_pointer += 1;
                }
                LESS_I32_REGISTER_ENCODED => {
                    let left_i32 = self.get_register(b_field as usize)?.as_value::<i32>();
                    let right_i32 = i32::decode(c_field)?;
                    let comparator = a_field != 0;
                    let jump_distance = ((left_i32 < right_i32) == comparator) as usize;

                    instruction_pointer += jump_distance + 1;
                }
                TEST_REGISTER => {
                    let value = self.get_register(b_field as usize)?.as_value::<bool>();
                    let comparator = a_field != 0;
                    let should_jump = !(value && comparator);
                    let jump_distance = c_field as usize * should_jump as usize;

                    instruction_pointer += jump_distance + 1;
                }
                TEST_ENCODED => {
                    let value = bool::decode(b_field)?;
                    let comparator = a_field != 0;
                    let should_jump = !(value && comparator);
                    let jump_distance = c_field as usize * should_jump as usize;

                    instruction_pointer += jump_distance + 1;
                }
                ADD_I32_REGISER_REGISTER => {
                    let left_i32 = self.get_register(b_field as usize)?.as_value::<i32>();
                    let right_i32 = self.get_register(c_field as usize)?.as_value::<i32>();
                    let sum = left_i32.saturating_add(right_i32);

                    self.set_register(a_field, sum)?;

                    instruction_pointer += 1;
                }
                ADD_I32_REGISTER_ENCODED => {
                    let left_i32 = self.get_register(b_field as usize)?.as_value::<i32>();
                    let right_i32 = i32::decode(c_field)?;
                    let sum = left_i32.saturating_add(right_i32);

                    self.set_register(a_field, sum)?;

                    instruction_pointer += 1;
                }
                MULTIPLY_I32_REGISTER_REGISTER => {
                    let left_i32 = self.get_register(b_field as usize)?.as_value::<i32>();
                    let right_i32 = self.get_register(c_field as usize)?.as_value::<i32>();
                    let product = left_i32.saturating_mul(right_i32);

                    self.set_register(a_field, product)?;

                    instruction_pointer += 1;
                }
                NEGATE_BOOLEAN_REGISTER => {
                    let value = self.get_register(b_field as usize)?.as_value::<bool>();
                    let negated = !value;

                    self.set_register(a_field, negated)?;

                    instruction_pointer += 1;
                }
                JUMP_FORWARD => {
                    instruction_pointer += a_field + 1;
                }
                JUMP_BACKWARD => {
                    instruction_pointer -= a_field + 1;
                }
                RETURN => {
                    self.call_stack.pop();
                }
                _ => {
                    return Err(VmError::InvalidInstructionDispatch { instruction });
                }
            }
        }

        Ok(Some(self.frame))
    }

    fn get_register(&self, index: usize) -> Result<&Register, VmError> {
        let absolute_index = self.frame.base_register as usize + index;

        if absolute_index >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: absolute_index,
            });
        }

        Ok(&self.register_stack[absolute_index])
    }

    fn get_double_registers(&self, index: usize) -> Result<(&Register, &Register), VmError> {
        let low_absolute_index = self.frame.base_register as usize + index;
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
        let low_absolute_index = self.frame.base_register as usize + index;
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
        let absolute_index = self.frame.base_register as usize + index;

        if absolute_index >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: absolute_index,
            });
        }

        self.register_stack[absolute_index] = Register::new(value);

        Ok(())
    }

    fn set_i64_to_registers(&mut self, index: usize, value: i64) -> Result<(), VmError> {
        let low_absolute_index = self.frame.base_register as usize + index;
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
        let low_absolute_index = self.frame.base_register as usize + index;
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
