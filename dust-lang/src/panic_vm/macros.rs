#![macro_use]

macro_rules! get_register {
    ($address: expr, $type: expr, $memory: expr, $call: expr) => {{
        use super::{Object, Register};
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
                assert!(
                    $address.index < $call.chunk.constants().len(),
                    "Constant index out of bounds: {} >= {}",
                    $address.index,
                    $call.chunk.constants().len()
                );

                let value = &$call.chunk.constants()[$address.index as usize];

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
                _ => unreachable!("Invalid operand type for encoding: {:?}", $type),
            },
            MemoryKind::CELL => todo!(),
            _ => unreachable!("Unsupported memory kind: {:?}", $address.memory),
        }
    }};
}

macro_rules! get_value_by_cloning_objects {
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
                    OperandType::INTEGER => Value::Integer(register.as_integer()),
                    OperandType::FLOAT => Value::Float(register.as_float()),
                    OperandType::BOOLEAN => Value::Boolean(register.as_boolean()),
                    OperandType::BYTE => Value::Byte(register.as_byte()),
                    OperandType::STRING => {
                        let object_index = register.as_index();
                        let object = $memory.objects[object_index];

                        if let Object::String(string) = object {
                            Value::String(string.clone())
                        } else {
                            panic!("Expected a string object, found: {:?}", object);
                        }
                    }
                    _ => todo!("Unsupported operand type: {:?}", $type),
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
                _ => unreachable!("Invalid operand type for encoding: {:?}", $type),
            },
            MemoryKind::CELL => todo!(),
            _ => unreachable!("Unsupported memory kind: {:?}", $address.memory),
        }
    }};
}

macro_rules! get_value_by_replacing_objects {
    ($address: expr, $type: expr, $memory: expr, $call: expr) => {{
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

                let register = $memory.registers[$address.index + $call.skipped_registers];

                match $type {
                    OperandType::INTEGER => Value::Integer(register.as_integer()),
                    OperandType::FLOAT => Value::Float(register.as_float()),
                    OperandType::BOOLEAN => Value::Boolean(register.as_boolean()),
                    OperandType::BYTE => Value::Byte(register.as_byte()),
                    OperandType::STRING => {
                        let object_index = register.as_index();
                        let object = replace(&mut $memory.objects[object_index], Object::Empty);

                        if let Object::String(string) = object {
                            Value::String(string)
                        } else {
                            panic!("Expected a string object, found: {:?}", object);
                        }
                    }
                    _ => todo!("Unsupported operand type: {:?}", $type),
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
                _ => unreachable!("Invalid operand type for encoding: {:?}", $type),
            },
            MemoryKind::CELL => todo!(),
            _ => unreachable!("Unsupported memory kind: {:?}", $address.memory),
        }
    }};
}
