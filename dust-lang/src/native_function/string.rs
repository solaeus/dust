use crate::{
    DustString,
    chunk::Arguments,
    instruction::{AddressKind, Destination},
    panic_vm::{CallFrame, Memory},
};

pub fn to_string(
    destination: Destination,
    arguments: &Arguments,
    call: &mut CallFrame,
    memory: &mut Memory,
) {
    let argument = &arguments.values[0];
    let stringified = match argument.kind {
        AddressKind::BOOLEAN_MEMORY => {
            let boolean = memory.booleans[argument.index as usize];

            DustString::from(boolean.to_string())
        }
        AddressKind::BOOLEAN_REGISTER => {
            let boolean = memory.registers.booleans[argument.index as usize];

            DustString::from(boolean.to_string())
        }
        AddressKind::BYTE_MEMORY => {
            let byte = memory.bytes[argument.index as usize];

            DustString::from(byte.to_string())
        }
        AddressKind::BYTE_REGISTER => {
            let byte = memory.registers.bytes[argument.index as usize];

            DustString::from(byte.to_string())
        }
        AddressKind::CHARACTER_CONSTANT => {
            let character = call.chunk.character_constants[argument.index as usize];

            DustString::from(character.to_string())
        }
        AddressKind::CHARACTER_MEMORY => {
            let character = memory.characters[argument.index as usize];

            DustString::from(character.to_string())
        }
        AddressKind::CHARACTER_REGISTER => {
            let character = memory.registers.characters[argument.index as usize];

            DustString::from(character.to_string())
        }
        AddressKind::FLOAT_CONSTANT => {
            let float = call.chunk.float_constants[argument.index as usize];

            DustString::from(float.to_string())
        }
        AddressKind::FLOAT_MEMORY => {
            let float = memory.floats[argument.index as usize];

            DustString::from(float.to_string())
        }
        AddressKind::FLOAT_REGISTER => {
            let float = memory.registers.floats[argument.index as usize];

            DustString::from(float.to_string())
        }
        AddressKind::INTEGER_CONSTANT => {
            let integer = call.chunk.integer_constants[argument.index as usize];

            DustString::from(integer.to_string())
        }
        AddressKind::INTEGER_MEMORY => {
            let integer = memory.integers[argument.index as usize];

            DustString::from(integer.to_string())
        }
        AddressKind::INTEGER_REGISTER => {
            let integer = memory.registers.integers[argument.index as usize];

            DustString::from(integer.to_string())
        }
        AddressKind::STRING_CONSTANT => {
            call.chunk.string_constants[argument.index as usize].clone()
        }
        AddressKind::STRING_MEMORY => memory.strings[argument.index as usize].clone(),
        AddressKind::STRING_REGISTER => memory.registers.strings[argument.index as usize].clone(),
        _ => todo!(),
    };

    if destination.is_register {
        memory.registers.strings[destination.index as usize] = stringified;
    } else {
        memory.strings[destination.index as usize] = stringified;
    }
}
