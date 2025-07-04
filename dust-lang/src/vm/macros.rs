#![macro_use]

macro_rules! read_register {
    ($index: expr,$memory: expr,  $call: expr, $operation: expr) => {{
        $memory
            .registers
            .get($index + $call.skipped_registers)
            .ok_or(RuntimeError($operation))?
    }};
}

macro_rules! read_register_mut {
    ($index: expr, $memory: expr, $call: expr, $operation: expr) => {{
        $memory
            .registers
            .get_mut($index + $call.skipped_registers)
            .ok_or(RuntimeError($operation))?
    }};
}

macro_rules! read_constant {
    ($index: expr, $call: expr, $operation: expr) => {{
        $call
            .chunk
            .constants()
            .get($index)
            .ok_or(RuntimeError($operation))?
    }};
}

macro_rules! read_object {
    ($index: expr,$memory: expr,  $operation: expr) => {{
        $memory
            .objects
            .get($index)
            .ok_or(RuntimeError($operation))?
    }};
}

macro_rules! get_register_from_address {
    ($address: expr, $type: expr, $memory: expr, $call: expr, $operation: expr) => {{
        use super::{Object, Register, RuntimeError};
        use crate::MemoryKind;

        match $address.memory {
            MemoryKind::REGISTER => {
                assert!(
                    $address.index < $memory.registers.len(),
                    "Register index out of bounds: {} >= {}",
                    $address.index,
                    $memory.registers.len()
                );

                $memory.registers[$address.index + $call.skipped_registers]
            }
            MemoryKind::CONSTANT => {
                let value = &$call
                    .chunk
                    .constants()
                    .get($address.index as usize)
                    .ok_or_else(|| RuntimeError($operation))?;

                match value {
                    Value::Integer(integer) => Register::integer(*integer),
                    Value::Float(float) => Register::float(*float),
                    Value::String(string) => {
                        let object = Object::String(string.clone());

                        $memory.store_object(object)
                    }
                    _ => todo!(),
                }
            }
            MemoryKind::ENCODED => match $type {
                OperandType::BOOLEAN => Register::boolean($address.index != 0),
                OperandType::BYTE => Register::byte($address.index as u8),
                _ => return Err(RuntimeError($operation)),
            },
            MemoryKind::CELL => todo!(),
            _ => unreachable!("Unsupported memory kind: {:?}", $address.memory),
        }
    }};
}

macro_rules! get_from_address_value_by_cloning_objects {
    ($address: expr, $type: expr, $memory: expr, $call: expr) => {{
        use super::Object;
        use crate::MemoryKind;

        match $address.memory {
            MemoryKind::REGISTER => {
                assert!(
                    $address.index < $memory.registers.len(),
                    "Register index out of bounds: {} >= {}",
                    $address.index,
                    $memory.registers.len()
                );

                let register = $memory.registers[$address.index + $call.skipped_registers];

                match $type {
                    OperandType::BOOLEAN => Value::Boolean(register.as_boolean()),
                    OperandType::BYTE => Value::Byte(register.as_byte()),
                    OperandType::INTEGER => Value::Integer(register.as_integer()),
                    OperandType::FLOAT => Value::Float(register.as_float()),
                    OperandType::STRING => {
                        let object_index = register.as_index();
                        let object = $memory.objects[object_index];

                        if let Object::String(string) = object {
                            Value::String(string.clone())
                        } else {
                            return Err(RuntimeError::InvalidObject($address));
                        }
                    }
                    _ => return Err(RuntimeError::InvalidOperandType($address)),
                }
            }
            MemoryKind::CONSTANT => {
                assert!(
                    $address.index < $call.chunk.constants().len(),
                    "Constant index out of bounds: {} >= {}",
                    $address.index,
                    $call.chunk.constants().len()
                );

                $call.chunk.constants()[$address.index as usize].clone()
            }
            MemoryKind::ENCODED => match $type {
                OperandType::BOOLEAN => Value::Boolean($address.index != 0),
                OperandType::BYTE => Value::Byte($address.index as u8),
                _ => return Err(RuntimeError::InvalidOperandType($type)),
            },
            MemoryKind::CELL => todo!(),
            _ => return Err(RuntimeError::InvalidAddress($address)),
        }
    }};
}

macro_rules! get_value_from_address_by_replacing_objects {
    ($address: expr, $type: expr, $memory: expr, $call: expr, $operation: expr) => {{
        use super::Object;
        use crate::MemoryKind;
        use std::mem::replace;

        match $address.memory {
            MemoryKind::REGISTER => {
                assert!(
                    $address.index < $memory.registers.len(),
                    "Register index out of bounds: {} >= {}",
                    $address.index,
                    $memory.registers.len()
                );

                let register = $memory
                    .registers
                    .get($address.index + $call.skipped_registers)
                    .ok_or_else(|| RuntimeError($operation))?;

                match $type {
                    OperandType::BOOLEAN => Value::Boolean(register.as_boolean()),
                    OperandType::BYTE => Value::Byte(register.as_byte()),
                    OperandType::INTEGER => Value::Integer(register.as_integer()),
                    OperandType::FLOAT => Value::Float(register.as_float()),
                    OperandType::STRING => {
                        let object_index = register.as_index();
                        let object = replace(&mut $memory.objects[object_index], Object::Empty);

                        if let Object::String(string) = object {
                            Value::String(string)
                        } else {
                            return Err(RuntimeError($operation));
                        }
                    }
                    _ => return Err(RuntimeError($operation)),
                }
            }
            MemoryKind::CONSTANT => {
                assert!(
                    $address.index < $call.chunk.constants().len(),
                    "Constant index out of bounds: {} >= {}",
                    $address.index,
                    $call.chunk.constants().len()
                );

                $call.chunk.constants()[$address.index as usize].clone()
            }
            MemoryKind::ENCODED => match $type {
                OperandType::BOOLEAN => Value::Boolean($address.index != 0),
                OperandType::BYTE => Value::Byte($address.index as u8),
                _ => return Err(RuntimeError($operation)),
            },
            MemoryKind::CELL => todo!(),
            _ => return Err(RuntimeError($operation)),
        }
    }};
}
