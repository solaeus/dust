#![macro_use]

macro_rules! get_integer {
    ($address: expr, $memory: expr, $call: expr, $operation: expr) => {{
        match $address.memory {
            MemoryKind::REGISTER => {
                read_register!($address.index, $memory, $call, $operation).as_integer()
            }
            MemoryKind::CONSTANT => read_constant!($address.index, $call, $operation)
                .as_integer()
                .ok_or(RuntimeError($operation))?,
            MemoryKind::CELL => todo!(),
            _ => return Err(RuntimeError($operation)),
        }
    }};
}

macro_rules! read_register {
    ($index: expr, $memory: expr, $call: expr, $operation: expr) => {{
        use tracing::trace;

        trace!(
            "Reading register at index: {}",
            $index + $call.skipped_registers
        );

        let total_index = $index + $call.skipped_registers;

        if total_index < $memory.top {
            $memory.stack[total_index]
        } else if total_index < $memory.top + $memory.heap.len() {
            $memory.heap[total_index - $memory.top]
        } else {
            return Err(RuntimeError($operation));
        }
    }};
}

macro_rules! read_register_mut {
    ($index: expr, $memory: expr, $call: expr, $operation: expr) => {{
        use tracing::trace;

        trace!(
            "Reading register mutably at index: {}",
            $index + $call.skipped_registers
        );

        let total_index = $index + $call.skipped_registers;

        if total_index < $memory.top {
            &mut $memory.stack[total_index]
        } else if total_index < $memory.top + $memory.heap.len() {
            &mut $memory.heap[total_index - $memory.top]
        } else {
            return Err(RuntimeError($operation));
        }
    }};
}

macro_rules! read_constant {
    ($index: expr, $call: expr, $operation: expr) => {{
        use tracing::trace;

        trace!("Reading constant at index: {}", $index);

        $call
            .chunk
            .constants()
            .get($index)
            .ok_or(RuntimeError($operation))?
    }};
}

macro_rules! read_object {
    ($index: expr, $memory: expr, $operation: expr) => {{
        use tracing::trace;

        trace!("Reading object at index: {}", $index);

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
            MemoryKind::REGISTER => read_register!($address.index, $memory, $call, $operation),
            MemoryKind::CONSTANT => {
                let value = &$call
                    .chunk
                    .constants()
                    .get($address.index as usize)
                    .ok_or_else(|| RuntimeError($operation))?;

                match value {
                    Value::Boolean(boolean) => Register::boolean(*boolean),
                    Value::Byte(byte) => Register::byte(*byte),
                    Value::Character(character) => Register::character(*character),
                    Value::Float(float) => Register::float(*float),
                    Value::Integer(integer) => Register::integer(*integer),
                    Value::String(string) => {
                        let object = Object::String(string.clone());

                        $memory.store_object(object)
                    }
                    Value::List(list) => {
                        let object = Object::ValueList(list.clone());

                        $memory.store_object(object)
                    }
                    Value::Function(function) => {
                        let object = Object::Function(function.clone());

                        $memory.store_object(object)
                    }
                }
            }
            MemoryKind::ENCODED => match $type {
                OperandType::BOOLEAN => Register::boolean($address.index != 0),
                OperandType::BYTE => Register::byte($address.index as u8),
                _ => return Err(RuntimeError($operation)),
            },
            MemoryKind::CELL => todo!(),
            _ => return Err(RuntimeError($operation)),
        }
    }};
}

macro_rules! get_value_from_address_by_cloning_objects {
    ($address: expr, $type: expr, $memory: expr, $call: expr, $operation: expr) => {{
        use super::Object;
        use crate::MemoryKind;

        match $address.memory {
            MemoryKind::REGISTER => {
                assert!(
                    $address.index < $memory.len(),
                    "Register index out of bounds: {} >= {}",
                    $address.index,
                    $memory.len()
                );

                let register = read_register!($address.index, $memory, $call, $operation);

                match $type {
                    OperandType::BOOLEAN => Value::Boolean(register.as_boolean()),
                    OperandType::BYTE => Value::Byte(register.as_byte()),
                    OperandType::INTEGER => Value::Integer(register.as_integer()),
                    OperandType::FLOAT => Value::Float(register.as_float()),
                    OperandType::STRING => {
                        let object_index = register.as_index();
                        let object = read_object!(object_index, $memory, $operation);

                        if let Object::String(string) = object {
                            Value::String(string.clone())
                        } else {
                            return Err(RuntimeError($operation));
                        }
                    }
                    OperandType::FUNCTION => {
                        let object_index = register.as_index();
                        let object = read_object!(object_index, $memory, $operation);

                        if let Object::Function(function) = object {
                            Value::Function(function.clone())
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

macro_rules! get_value_from_address_by_replacing_objects {
    ($address: expr, $type: expr, $memory: expr, $call: expr, $operation: expr) => {{
        use super::Object;
        use crate::MemoryKind;
        use std::mem::replace;

        match $address.memory {
            MemoryKind::REGISTER => {
                assert!(
                    $address.index < $memory.len(),
                    "Register index out of bounds: {} >= {}",
                    $address.index,
                    $memory.len()
                );

                let register = $memory
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
                    OperandType::LIST_BOOLEAN
                    | OperandType::LIST_BYTE
                    | OperandType::LIST_CHARACTER
                    | OperandType::LIST_FLOAT
                    | OperandType::LIST_INTEGER => {
                        let object_index = register.as_index();
                        let object = replace(&mut $memory.objects[object_index], Object::Empty);

                        if let Object::ValueList(list) = object {
                            Value::List(list)
                        } else {
                            return Err(RuntimeError($operation));
                        }
                    }
                    _ => todo!(),
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
