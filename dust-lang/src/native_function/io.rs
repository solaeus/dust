use std::io::{Write, stdout};

use crate::{
    DustString, Instruction, Type, instruction::CallNative, risky_vm::Thread, r#type::TypeKind,
};

pub fn read_line(instruction: Instruction, thread: &mut Thread) {
    let CallNative { destination, .. } = CallNative::from(instruction);
    let mut buffer = String::new();
    let stdin = std::io::stdin();

    stdin.read_line(&mut buffer).unwrap();

    *thread
        .current_memory
        .strings
        .get_mut(destination.index as usize)
        .unwrap()
        .as_value_mut() = DustString::from(buffer.trim_end_matches('\n'));
}

pub fn write_line(instruction: Instruction, thread: &mut Thread) {
    let CallNative {
        destination: _,
        function,
        argument_list_index,
    } = CallNative::from(instruction);

    let arguments = thread
        .current_call
        .chunk
        .arguments
        .get(argument_list_index as usize)
        .unwrap();
    let mut stdout = stdout();

    for address in &arguments.values {
        match address.r#type() {
            TypeKind::String => {
                let string = thread.resolve_string(address);

                stdout.write_all(string.as_bytes()).unwrap();
            }
            _ => unreachable!(),
        }
    }

    stdout.write_all(b"\n").unwrap();
}
