use std::io::{Write, stdin, stdout};

use crate::{
    Arguments, Destination, DustString,
    panic_vm::{CallFrame, Memory},
    r#type::TypeKind,
};

pub fn read_line<const REGISTER_COUNT: usize>(
    destination: Destination,
    _: &Arguments,
    _: &mut CallFrame,
    memory: &mut Memory<REGISTER_COUNT>,
) {
    let mut buffer = String::new();
    let _ = stdin().read_line(&mut buffer);
    let string = DustString::from(buffer.trim_end_matches('\n'));

    memory.set_string(destination.is_register, destination.index, string);
}

pub fn write_line<const REGISTER_COUNT: usize>(
    _: Destination,
    arguments: &Arguments,
    _: &mut CallFrame,
    memory: &mut Memory<REGISTER_COUNT>,
) {
    let mut stdout = stdout();

    for address in &arguments.values {
        match address.kind.r#type() {
            TypeKind::Boolean => {
                let boolean = memory.get_boolean(address.is_register(), address.index);
                let _ = stdout.write_all(boolean.to_string().as_bytes());
            }
            TypeKind::Byte => {
                let byte = memory.get_byte(address.is_register(), address.index);
                let _ = stdout.write_all(byte.to_string().as_bytes());
            }
            TypeKind::Character => {
                let character = memory.get_character(address.is_register(), address.index);
                let _ = stdout.write_all(character.to_string().as_bytes());
            }
            TypeKind::Float => {
                let float = memory.get_float(address.is_register(), address.index);
                let _ = stdout.write_all(float.to_string().as_bytes());
            }
            TypeKind::Integer => {
                let integer = memory.get_integer(address.is_register(), address.index);
                let _ = stdout.write_all(integer.to_string().as_bytes());
            }
            TypeKind::String => {
                let string = memory.get_string(address.is_register(), address.index);
                let _ = stdout.write_all(string.as_str().as_bytes());
            }
            TypeKind::List => {
                let list = memory.get_list(address.is_register(), address.index);
                let concrete_list = memory.make_list_concrete(list);
                let _ = stdout.write_all(concrete_list.to_string().as_bytes());
            }
            TypeKind::Function => {
                let function = memory.get_function(address.is_register(), address.index);
                let _ = stdout.write_all(function.to_string().as_bytes());
            }
            _ => unreachable!(),
        }
    }

    let _ = stdout.write_all(b"\n");
}
