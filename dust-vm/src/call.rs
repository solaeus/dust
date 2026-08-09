use dust_compiler::{constants::Constants, instruction::Instruction, prototype::Prototype};
use dust_keys::{create_operand_dispatch, create_operation_dispatch};

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

    pub fn run(mut self) -> Result<Option<Vec<Register>>, VmError> {
        let prototype = self
            .prototypes
            .get(self.frame.prototype_index as usize)
            .ok_or(VmError::InvalidPrototypeIndex {
                index: self.frame.prototype_index,
            })?;
        let mut instruction_pointer = self.frame.instruction_pointer as usize;
        let mut operands = Operands::default();

        loop {
            let instruction = *prototype.instructions.get(instruction_pointer).ok_or(
                VmError::InvalidInstructionDispatch {
                    index: instruction_pointer,
                },
            )?;
            let operand_handler = create_operand_dispatch!(
                (instruction.operand_key()),
                BOOLEAN REGISTER EMPTY => Call::get_left_register::<bool>,
                I_8 REGISTER EMPTY => Call::get_left_register::<i8>,
                I_16 REGISTER EMPTY => Call::get_left_register::<i16>,
                I_32 REGISTER EMPTY => Call::get_left_register::<i32>,
                I_64 REGISTER EMPTY => Call::get_left_register::<i64>,
                I_128 REGISTER EMPTY => Call::get_left_register::<i128>,
                BOOLEAN ENCODED EMPTY => Call::get_left_encoded::<bool>,
                BOOLEAN ENCODED ENCODED => Call::get_both_encoded::<bool>,
                * * * => Call::emit_operand_error,
            );
            let operation_handler = create_operation_dispatch!(
                (instruction.operation_key()),
                MOVE BOOLEAN => Call::move_boolean,
                MOVE I_8 => Call::move_i8,
                MOVE I_16 => Call::move_i16,
                MOVE I_32 => Call::move_i32,
                MOVE I_64 => Call::move_i64,
                MOVE I_128 => Call::move_i128,
                * * => Call::emit_operation_error,
            );

            operand_handler(
                &mut self,
                instruction.a_field_as_u64(),
                instruction.b_field_as_u64(),
                &mut operands,
            )?;
            operation_handler(
                &mut self,
                instruction.a_field_as_usize(),
                &mut operands,
                &mut instruction_pointer,
            )?;
        }
    }

    fn move_boolean(
        &mut self,
        destination: usize,
        operands: &mut Operands,
        instruction_pointer: &mut usize,
    ) -> Result<(), VmError> {
        let operand = operands.boolean.0;

        self.set_register(destination, Register::new(operand))?;

        *instruction_pointer += 1;

        Ok(())
    }

    fn move_i8(
        &mut self,
        destination: usize,
        operands: &mut Operands,
        instruction_pointer: &mut usize,
    ) -> Result<(), VmError> {
        let operand = operands.i8.0;

        self.set_register(destination, Register::new(operand))?;

        *instruction_pointer += 1;

        Ok(())
    }

    fn move_i16(
        &mut self,
        destination: usize,
        operands: &mut Operands,
        instruction_pointer: &mut usize,
    ) -> Result<(), VmError> {
        let operand = operands.i16.0;

        self.set_register(destination, Register::new(operand))?;

        *instruction_pointer += 1;

        Ok(())
    }

    fn move_i32(
        &mut self,
        destination: usize,
        operands: &mut Operands,
        instruction_pointer: &mut usize,
    ) -> Result<(), VmError> {
        let operand = operands.i32.0;

        self.set_register(destination, Register::new(operand))?;

        *instruction_pointer += 1;

        Ok(())
    }

    fn move_i64(
        &mut self,
        destination: usize,
        operands: &mut Operands,
        instruction_pointer: &mut usize,
    ) -> Result<(), VmError> {
        let operand = operands.i64.0;

        self.set_i64_to_registers(destination, operand)?;

        *instruction_pointer += 1;

        Ok(())
    }

    fn move_i128(
        &mut self,
        destination: usize,
        operands: &mut Operands,
        instruction_pointer: &mut usize,
    ) -> Result<(), VmError> {
        let operand = operands.i128.0;

        self.set_i128_to_registers(destination, operand)?;

        *instruction_pointer += 1;

        Ok(())
    }

    fn add_i32_i32(
        &mut self,
        destination: usize,
        operands: &mut Operands,
        instruction_pointer: &mut usize,
    ) -> Result<(), VmError> {
        let (left, right) = operands.i32;
        let sum = left.saturating_add(right);

        self.set_register(destination, Register::new(sum))?;

        *instruction_pointer += 1;

        Ok(())
    }

    fn emit_operand_error(
        &mut self,
        _left_index: u64,
        _right_index: u64,
        _operands: &mut Operands,
    ) -> Result<(), VmError> {
        Err(VmError::InvalidInstructionPointer {
            instruction_pointer: self.frame.instruction_pointer,
        })
    }

    fn emit_operation_error(
        &mut self,
        _destination: usize,
        _operands: &mut Operands,
        _instruction_pointer: &mut usize,
    ) -> Result<(), VmError> {
        Err(VmError::InvalidInstructionPointer {
            instruction_pointer: self.frame.instruction_pointer,
        })
    }

    fn get_left_register<T: Operand>(
        &mut self,
        left_index: u64,
        _right_index: u64,
        operands: &mut Operands,
    ) -> Result<(), VmError> {
        let register = self.get_register(left_index as usize)?;

        T::get_from_register(register, operands)
    }

    fn get_left_encoded<T: Operand>(
        &mut self,
        left_index: u64,
        _right_index: u64,
        operands: &mut Operands,
    ) -> Result<(), VmError> {
        T::get_encoded(left_index, operands)
    }

    fn get_both_encoded<T: Operand>(
        &mut self,
        left_index: u64,
        right_index: u64,
        operands: &mut Operands,
    ) -> Result<(), VmError> {
        T::get_encoded(left_index, operands)?;
        T::get_encoded(right_index, operands)
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

    fn set_register(&mut self, index: usize, register: Register) -> Result<(), VmError> {
        let absolute_index = self.frame.base_register as usize + index;

        if absolute_index >= self.register_stack.len() {
            return Err(VmError::InvalidRegisterIndex {
                index: absolute_index,
            });
        }

        self.register_stack[absolute_index] = register;

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
    pub instruction_pointer: u32,
}

#[derive(Default)]
struct Operands {
    boolean: (bool, bool),
    i8: (i8, i8),
    i16: (i16, i16),
    i32: (i32, i32),
    i64: (i64, i64),
    i128: (i128, i128),
    u8: (u8, u8),
    u16: (u16, u16),
    u32: (u32, u32),
    u64: (u64, u64),
    u128: (u128, u128),
    f32: (f32, f32),
    f64: (f64, f64),
    character: (char, char),
    function: (u32, u32),
    heap_pointer: (*const u8, *const u8),
}

trait Operand {
    fn get_encoded(index: u64, operands: &mut Operands) -> Result<(), VmError> {
        Err(VmError::InvalidOperandDispatch)
    }

    fn get_from_register(register: &Register, operands: &mut Operands) -> Result<(), VmError> {
        Err(VmError::InvalidOperandDispatch)
    }

    fn get_from_double_registers(
        low: &Register,
        high: &Register,
        operands: &mut Operands,
    ) -> Result<(), VmError> {
        Err(VmError::InvalidOperandDispatch)
    }

    fn get_from_quad_registers(
        low: &Register,
        low_mid: &Register,
        high_mid: &Register,
        high: &Register,
        operands: &mut Operands,
    ) -> Result<(), VmError> {
        Err(VmError::InvalidOperandDispatch)
    }
}

impl Operand for bool {
    fn get_encoded(index: u64, operands: &mut Operands) -> Result<(), VmError> {
        operands.boolean.0 = index != 0;

        Ok(())
    }

    fn get_from_register(register: &Register, operands: &mut Operands) -> Result<(), VmError> {
        operands.boolean.0 = register.as_value::<bool>();

        Ok(())
    }
}

impl Operand for i8 {
    fn get_encoded(index: u64, operands: &mut Operands) -> Result<(), VmError> {
        operands.i8.0 = index as i8;

        Ok(())
    }

    fn get_from_register(register: &Register, operands: &mut Operands) -> Result<(), VmError> {
        operands.i8.0 = register.as_value::<i8>();

        Ok(())
    }
}

impl Operand for i16 {
    fn get_encoded(index: u64, operands: &mut Operands) -> Result<(), VmError> {
        operands.i16.0 = index as i16;

        Ok(())
    }

    fn get_from_register(register: &Register, operands: &mut Operands) -> Result<(), VmError> {
        operands.i16.0 = register.as_value::<i16>();

        Ok(())
    }
}

impl Operand for i32 {
    fn get_encoded(index: u64, operands: &mut Operands) -> Result<(), VmError> {
        operands.i32.0 = index as i32;

        Ok(())
    }

    fn get_from_register(register: &Register, operands: &mut Operands) -> Result<(), VmError> {
        operands.i32.0 = register.as_value::<i32>();

        Ok(())
    }
}

impl Operand for i64 {
    fn get_encoded(index: u64, operands: &mut Operands) -> Result<(), VmError> {
        operands.i64.0 = index as i64;

        Ok(())
    }

    fn get_from_double_registers(
        low: &Register,
        high: &Register,
        operands: &mut Operands,
    ) -> Result<(), VmError> {
        let low_value = low.as_value::<u32>() as u64;
        let high_value = high.as_value::<u32>() as u64;

        operands.i64.0 = ((high_value << 32) | low_value) as i64;

        Ok(())
    }
}

impl Operand for i128 {
    fn get_encoded(index: u64, operands: &mut Operands) -> Result<(), VmError> {
        operands.i128.0 = index as i128;

        Ok(())
    }

    fn get_from_quad_registers(
        low: &Register,
        low_mid: &Register,
        high_mid: &Register,
        high: &Register,
        operands: &mut Operands,
    ) -> Result<(), VmError> {
        let low_value = low.as_value::<u32>() as u128;
        let low_mid_value = low_mid.as_value::<u32>() as u128;
        let high_mid_value = high_mid.as_value::<u32>() as u128;
        let high_value = high.as_value::<u32>() as u128;

        operands.i128.0 =
            ((high_value << 96) | (high_mid_value << 64) | (low_mid_value << 32) | low_value)
                as i128;

        Ok(())
    }
}
