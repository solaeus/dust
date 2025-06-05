use crate::{
    Address, DustString,
    instruction::{AddressKind, Destination},
    panic_vm::{CallFrame, Memory, ThreadPool, macros::*},
};

pub fn to_string<const REGISTER_COUNT: usize>(
    destination: Destination,
    arguments: &[Address],
    call: &mut CallFrame,
    memory: &mut Memory<REGISTER_COUNT>,
    _: &ThreadPool<REGISTER_COUNT>,
) {
    let stringified = match arguments[0].kind {
        AddressKind::BOOLEAN_MEMORY => get_memory!(memory, booleans, arguments[0]).to_string(),
        AddressKind::BOOLEAN_REGISTER => get_register!(memory, booleans, arguments[0]).to_string(),
        AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, arguments[0]).to_string(),
        AddressKind::BYTE_REGISTER => get_register!(memory, bytes, arguments[0]).to_string(),
        AddressKind::CHARACTER_CONSTANT => {
            get_constant!(call.chunk, character_constants, arguments[0]).to_string()
        }
        AddressKind::CHARACTER_MEMORY => get_memory!(memory, characters, arguments[0]).to_string(),
        AddressKind::CHARACTER_REGISTER => {
            get_register!(memory, characters, arguments[0]).to_string()
        }
        AddressKind::FLOAT_CONSTANT => {
            get_constant!(call.chunk, float_constants, arguments[0]).to_string()
        }
        AddressKind::FLOAT_MEMORY => get_memory!(memory, floats, arguments[0]).to_string(),
        AddressKind::FLOAT_REGISTER => get_register!(memory, floats, arguments[0]).to_string(),
        AddressKind::INTEGER_CONSTANT => {
            get_constant!(call.chunk, integer_constants, arguments[0]).to_string()
        }
        AddressKind::INTEGER_MEMORY => get_memory!(memory, integers, arguments[0]).to_string(),
        AddressKind::INTEGER_REGISTER => get_register!(memory, integers, arguments[0]).to_string(),
        AddressKind::STRING_CONSTANT => {
            get_constant!(call.chunk, string_constants, arguments[0]).to_string()
        }
        AddressKind::STRING_MEMORY => get_memory!(memory, strings, arguments[0]).to_string(),
        AddressKind::STRING_REGISTER => get_register!(memory, strings, arguments[0]).to_string(),
        AddressKind::LIST_MEMORY => {
            let abstract_list = get_memory!(memory, lists, arguments[0]);

            memory.make_list_concrete(abstract_list).to_string()
        }
        AddressKind::LIST_REGISTER => {
            let abstract_list = get_register!(memory, lists, arguments[0]);

            memory.make_list_concrete(abstract_list).to_string()
        }
        AddressKind::FUNCTION_PROTOTYPE => {
            get_constant!(call.chunk, prototypes, arguments[0]).to_string()
        }
        AddressKind::FUNCTION_SELF => call.chunk.to_string(),
        AddressKind::FUNCTION_REGISTER => {
            get_register!(memory, functions, arguments[0]).to_string()
        }
        AddressKind::FUNCTION_MEMORY => get_memory!(memory, functions, arguments[0]).to_string(),
        _ => unreachable!(),
    };
    let string = DustString::from(stringified);

    set!(memory, strings, destination, string);
}
