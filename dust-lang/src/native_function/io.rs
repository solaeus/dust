use std::io::{Write, stdout};

use crate::{DustString, Instruction, Type, instruction::CallNative, risky_vm::Thread};

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
        .argument_lists
        .get(argument_list_index as usize)
        .unwrap();
    let mut stdout = stdout();

    for ((argument_index, _), argument_type) in arguments
        .0
        .iter()
        .zip(function.r#type().value_parameters.iter())
    {
        match argument_type {
            Type::String => {
                let string = thread
                    .current_memory
                    .strings
                    .get(*argument_index as usize)
                    .unwrap()
                    .as_value();

                stdout.write_all(string.as_bytes()).unwrap();
            }
            _ => unreachable!(),
        }
    }

    stdout.write_all(b"\n").unwrap();
}
