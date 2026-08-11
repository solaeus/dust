use dust_compiler::{constants::Constants, instruction::dispatch_keys, prototype::Prototype};

use crate::{
    error::VmError,
    register::{Register, RegisterValue},
};

pub struct Call<'a> {
    prototypes: &'a [Prototype],

    register_stack: &'a mut Vec<Register>,

    call_stack: &'a mut Vec<CallFrame>,

    constants: &'a Constants,

    base_register: usize,
}

impl<'a> Call<'a> {
    pub fn new(
        prototypes: &'a [Prototype],
        constants: &'a Constants,
        register_stack: &'a mut Vec<Register>,
        call_stack: &'a mut Vec<CallFrame>,
    ) -> Result<Self, VmError> {
        Ok(Self {
            prototypes,
            register_stack,
            constants,
            call_stack,
            base_register: 0,
        })
    }

    pub fn run(mut self) -> Result<Option<CallFrame>, VmError> {
        let mut current_frame = *self.call_stack.last().ok_or(VmError::CallStackUnderflow)?;

        self.base_register = current_frame.base_register as usize;

        let mut current_prototype = self
            .prototypes
            .get(current_frame.prototype_index as usize)
            .ok_or(VmError::InvalidPrototypeIndex {
                index: current_frame.prototype_index,
            })?;
        let mut instruction_pointer = current_frame.instruction_pointer as usize;

        while !self.call_stack.is_empty() {
            let instruction = *current_prototype
                .instructions
                .get(instruction_pointer)
                .ok_or(VmError::InvalidInstructionPointer {
                    instruction_pointer,
                })?;
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
                MOVE_FUNCTION_ENCODED => {
                    let prototype_index = b_field as u32;

                    self.set_register(a_field, prototype_index)?;

                    instruction_pointer += c_field as usize + 1;
                }
                GET_BOOLEAN_REGISTER => {
                    let destination = self.base_register + a_field as usize;
                    let base_register = self.base_register + b_field as usize;
                    let offset = self.get_register(c_field as usize)?.as_bits() as usize;
                    let source_start = base_register + offset;
                    let source_end = source_start + 1;

                    self.register_stack
                        .copy_within(source_start..source_end, destination);

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
                LESS_EQUAL_REGISTER_ENCODED => {
                    let left_i32 = self.get_register(b_field as usize)?.as_value::<i32>();
                    let right_i32 = i32::decode(c_field)?;
                    let comparator = a_field != 0;
                    let jump_distance = ((left_i32 <= right_i32) == comparator) as usize;

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
                SUBTRACT_I32_REGISTER_ENCODED => {
                    let left_i32 = self.get_register(b_field as usize)?.as_value::<i32>();
                    let right_i32 = i32::decode(c_field)?;
                    let difference = left_i32.saturating_sub(right_i32);

                    self.set_register(a_field, difference)?;

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
                CALL_REGISTER => {
                    let next_prototype_index = self.get_register(b_field as usize)?.as_bits();
                    let next_prototype = self.prototypes.get(next_prototype_index as usize).ok_or(
                        VmError::InvalidPrototypeIndex {
                            index: b_field as u32,
                        },
                    )?;
                    let next_frame = CallFrame {
                        prototype_index: next_prototype_index,
                        base_register: self.base_register as u32 + c_field as u32,
                        register_count: next_prototype.register_count,
                        return_register_count: next_prototype.return_types.len() as u16,
                        instruction_pointer: 0,
                    };
                    let current_frame_mut = self
                        .call_stack
                        .last_mut()
                        .ok_or(VmError::CallStackUnderflow)?;
                    current_frame_mut.instruction_pointer = instruction_pointer as u32 + 1;

                    self.call_stack.push(next_frame);

                    current_frame = next_frame;
                    current_prototype = next_prototype;
                    instruction_pointer = 0;
                    self.base_register = current_frame.base_register as usize;
                }
                CALL_ENCODED => {
                    let next_prototype = self.prototypes.get(b_field as usize).ok_or(
                        VmError::InvalidPrototypeIndex {
                            index: b_field as u32,
                        },
                    )?;
                    let next_frame = CallFrame {
                        prototype_index: b_field as u32,
                        base_register: self.base_register as u32 + c_field as u32,
                        register_count: next_prototype.register_count,
                        return_register_count: next_prototype.return_types.len() as u16,
                        instruction_pointer: 0,
                    };
                    let current_frame_mut = self
                        .call_stack
                        .last_mut()
                        .ok_or(VmError::CallStackUnderflow)?;
                    current_frame_mut.instruction_pointer = instruction_pointer as u32 + 1;

                    self.call_stack.push(next_frame);

                    current_frame = next_frame;
                    current_prototype = next_prototype;
                    instruction_pointer = 0;
                    self.base_register = current_frame.base_register as usize;
                }
                JUMP_FORWARD => {
                    instruction_pointer += a_field + 1;
                }
                JUMP_BACKWARD => {
                    instruction_pointer -= a_field + 1;
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

                    current_frame = next_frame;
                    current_prototype = next_prototype;
                    instruction_pointer = current_frame.instruction_pointer as usize;
                    self.base_register = current_frame.base_register as usize;
                }
                _ => {
                    return Err(VmError::InvalidInstructionDispatch { instruction });
                }
            }
        }

        Ok(self.call_stack.pop())
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
