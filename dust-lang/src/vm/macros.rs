#![macro_use]

macro_rules! get_boolean {
    ($address: expr, $memory: expr, $call: expr, $operation: expr) => {{
        use tracing::trace;

        trace!("Reading boolean at address: {:?}", $address);

        match $address.memory {
            MemoryKind::ENCODED => $address.index != 0,
            MemoryKind::REGISTER => {
                read_register!($address.index, $memory, $call, $operation).as_boolean()
            }
            MemoryKind::CELL => todo!(),
            _ => return Err(RuntimeError($operation)),
        }
    }};
}

macro_rules! get_byte {
    ($address: expr, $memory: expr, $call: expr, $operation: expr) => {{
        use tracing::trace;

        trace!("Reading byte at address: {:?}", $address);

        match $address.memory {
            MemoryKind::ENCODED => $address.index as u8,
            MemoryKind::REGISTER => {
                read_register!($address.index, $memory, $call, $operation).as_byte()
            }
            MemoryKind::CELL => todo!(),
            _ => return Err(RuntimeError($operation)),
        }
    }};
}

macro_rules! get_character {
    ($address: expr, $memory: expr, $call: expr, $operation: expr) => {{
        use tracing::trace;

        trace!("Reading character at address: {:?}", $address);

        match $address.memory {
            MemoryKind::REGISTER => {
                read_register!($address.index, $memory, $call, $operation).as_character()
            }
            MemoryKind::CONSTANT => read_constant!($address.index, $call, $operation)
                .as_character()
                .ok_or(RuntimeError($operation))?,
            MemoryKind::CELL => todo!(),
            _ => return Err(RuntimeError($operation)),
        }
    }};
}

macro_rules! get_float {
    ($address: expr, $memory: expr, $call: expr, $operation: expr) => {{
        match $address.memory {
            MemoryKind::REGISTER => {
                read_register!($address.index, $memory, $call, $operation).as_float()
            }
            MemoryKind::CONSTANT => read_constant!($address.index, $call, $operation)
                .as_float()
                .ok_or(RuntimeError($operation))?,
            MemoryKind::CELL => todo!(),
            _ => return Err(RuntimeError($operation)),
        }
    }};
}

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

macro_rules! get_string {
    ($address: expr, $memory: expr, $call: expr, $operation: expr) => {{
        use tracing::trace;

        trace!("Reading string at address: {:?}", $address);

        match $address.memory {
            MemoryKind::REGISTER => {
                let register = read_register!($address.index, $memory, $call, $operation);
                let object_index = register.as_index();
                let object = read_object!(object_index, $memory, $operation);

                if let Object::String(string) = object {
                    string
                } else {
                    return Err(RuntimeError($operation));
                }
            }
            MemoryKind::CONSTANT => read_constant!($address.index, $call, $operation)
                .as_string()
                .ok_or(RuntimeError($operation))?,
            MemoryKind::CELL => todo!(),
            _ => return Err(RuntimeError($operation)),
        }
    }};
}

macro_rules! get_list {
    ($address: expr, $memory: expr, $call: expr, $operation: expr) => {{
        {
            use tracing::trace;

            trace!("Reading list at address: {:?}", $address);

            match $address.memory {
                MemoryKind::REGISTER => {
                    let register = read_register!($address.index, $memory, $call, $operation);
                    let object_index = register.as_index();
                    let object = read_object!(object_index, $memory, $operation);

                    if let Object::ValueList(list) = object {
                        list
                    } else {
                        return Err(RuntimeError($operation));
                    }
                }
                MemoryKind::CONSTANT => read_constant!($address.index, $call, $operation)
                    .as_list()
                    .ok_or(RuntimeError($operation))?,
                MemoryKind::CELL => todo!(),
                _ => return Err(RuntimeError($operation)),
            }
        }
    }};
}

macro_rules! get_function {
    ($address: expr, $memory: expr, $call: expr, $operation: expr) => {{
        use tracing::trace;

        trace!("Reading function at address: {:?}", $address);

        match $address.memory {
            MemoryKind::REGISTER => {
                let register = read_register!($address.index, $memory, $call, $operation);
                let object_index = register.as_index();
                let object = read_object!(object_index, $memory, $operation);

                if let Object::Function(function) = object {
                    function
                } else {
                    return Err(RuntimeError($operation));
                }
            }
            MemoryKind::CONSTANT => read_constant!($address.index, $call, $operation)
                .as_function()
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
            trace!("Reading from stack at index: {}", total_index);

            $memory.stack[total_index]
        } else if total_index < $memory.top + $memory.heap.len() {
            trace!("Reading from heap at index: {}", total_index - $memory.top);

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
            trace!("Reading from stack mutably at index: {}", total_index);

            &mut $memory.stack[total_index]
        } else if total_index < $memory.top + $memory.heap.len() {
            trace!(
                "Reading from heap mutably at index: {}",
                total_index - $memory.top
            );

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
            MemoryKind::CONSTANT => match $type {
                OperandType::BOOLEAN => Register::boolean($address.index != 0),
                OperandType::BYTE => Register::byte($address.index as u8),
                _ => {
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
            },
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

macro_rules! get_value_from_address_by_replacing_objects {
    (
        $address: expr,
        $type: expr,
        $memory: expr,
        $call: expr,
        $operation: expr,
    ) => {{
        match $address.memory {
            MemoryKind::REGISTER => {
                let register = read_register!($address.index, $memory, $call, $operation);

                match $type {
                    OperandType::BOOLEAN => Value::Boolean(register.as_boolean()),
                    OperandType::BYTE => Value::Byte(register.as_byte()),
                    OperandType::CHARACTER => Value::Character(register.as_character()),
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
                    | OperandType::LIST_INTEGER
                    | OperandType::LIST_STRING
                    | OperandType::LIST_LIST
                    | OperandType::LIST_FUNCTION => {
                        let object_index = register.as_index();
                        let object = replace(&mut $memory.objects[object_index], Object::Empty);

                        if let Object::ValueList(list) = object {
                            Value::List(list)
                        } else {
                            return Err(RuntimeError($operation));
                        }
                    }
                    OperandType::FUNCTION => {
                        let object_index = register.as_index();
                        let object = replace(&mut $memory.objects[object_index], Object::Empty);

                        if let Object::Function(function) = object {
                            Value::Function(function)
                        } else {
                            return Err(RuntimeError($operation));
                        }
                    }
                    _ => todo!(),
                }
            }
            MemoryKind::CONSTANT => read_constant!($address.index, $call, $operation).clone(),
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
