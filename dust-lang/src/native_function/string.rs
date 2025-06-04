use crate::{
    chunk::Arguments,
    instruction::Destination,
    panic_vm::{CallFrame, Memory},
    r#type::TypeKind,
};

pub fn to_string<const REGISTER_COUNT: usize>(
    destination: Destination,
    arguments: &Arguments,
    call: &mut CallFrame,
    memory: &mut Memory<REGISTER_COUNT>,
) {
    let argument = &arguments.values[0];
    let stringified = match argument.kind.r#type() {
        TypeKind::Boolean => memory
            .get_boolean(argument.is_register(), argument.index)
            .to_string()
            .into(),
        TypeKind::Byte => memory
            .get_byte(argument.is_register(), argument.index)
            .to_string()
            .into(),
        TypeKind::Character => {
            if argument.is_constant() {
                call.chunk.character_constants[argument.index as usize]
                    .to_string()
                    .into()
            } else {
                memory
                    .get_character(argument.is_register(), argument.index)
                    .to_string()
                    .into()
            }
        }
        TypeKind::Float => {
            if argument.is_constant() {
                call.chunk.float_constants[argument.index as usize]
                    .to_string()
                    .into()
            } else {
                memory
                    .get_float(argument.is_register(), argument.index)
                    .to_string()
                    .into()
            }
        }
        TypeKind::Integer => {
            if argument.is_constant() {
                call.chunk.integer_constants[argument.index as usize]
                    .to_string()
                    .into()
            } else {
                memory
                    .get_integer(argument.is_register(), argument.index)
                    .to_string()
                    .into()
            }
        }
        TypeKind::String => {
            if argument.is_constant() {
                call.chunk.string_constants[argument.index as usize].clone()
            } else {
                memory
                    .get_string(argument.is_register(), argument.index)
                    .clone()
            }
        }
        TypeKind::List => {
            let list = memory.get_list(argument.is_register(), argument.index);
            memory.make_list_concrete(list).to_string().into()
        }
        TypeKind::Function => {
            let function = memory.get_function(argument.is_register(), argument.index);
            function.to_string().into()
        }

        _ => todo!(),
    };

    memory.set_string(destination.is_register, destination.index, stringified);
}
