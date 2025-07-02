#![macro_use]

macro_rules! get_register {
    ($address: expr, $type: expr, $memory: expr, $skipped_registers: expr, $constants: expr) => {{
        use crate::MemoryKind;

        match $address.memory {
            MemoryKind::REGISTER => {
                assert!(
                    $address.index < $memory.registers.len(),
                    "Register index out of bounds: {} >= {}",
                    $address.index,
                    $memory.registers.len()
                );

                $memory.registers[$address.index + $skipped_registers]
            }
            MemoryKind::CONSTANT => {
                let value = &$constants[$address.index as usize];

                match value {
                    Value::Integer(integer) => Register::integer(*integer),
                    Value::Float(float) => Register::float(*float),
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

macro_rules! get_value {
    ($address: expr, $type: expr, $memory: expr, $skipped_registers: expr, $constants: expr) => {{
        use crate::MemoryKind;

        match $address.memory {
            MemoryKind::REGISTER => {
                assert!(
                    $address.index < $memory.registers.len(),
                    "Register index out of bounds: {} >= {}",
                    $address.index,
                    $memory.registers.len()
                );

                let register = $memory.registers[$address.index + $skipped_registers];

                match $type {
                    OperandType::INTEGER => Value::Integer(register.as_integer()),
                    OperandType::FLOAT => Value::Float(register.as_float()),
                    OperandType::BOOLEAN => Value::Boolean(register.as_boolean()),
                    OperandType::BYTE => Value::Byte(register.as_byte()),
                    _ => todo!("Unsupported operand type: {:?}", $type),
                }
            }
            MemoryKind::CONSTANT => {
                let value = &$constants[$address.index as usize];

                value.clone()
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
