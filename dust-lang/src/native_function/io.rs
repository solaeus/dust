use std::io::{Write, stdout};

use crate::{DustString, Instruction, Type, instruction::CallNative, risky_vm::Thread};

pub fn read_line(instruction: Instruction, thread: &mut Thread) {
    let CallNative { destination, .. } = CallNative::from(instruction);
    let mut buffer = String::new();
    let stdin = std::io::stdin();

    stdin.read_line(&mut buffer).unwrap();

    *thread
        .current_registers_mut()
        .strings
        .get_mut(destination as usize)
        .as_value_mut() = DustString::from(buffer.trim_end_matches('\n'));
}

pub fn write_line(instruction: Instruction, thread: &mut Thread) {
    let CallNative {
        destination: _,
        function,
        argument_list_index,
    } = CallNative::from(instruction);

    let current_frame = thread.current_frame();
    let current_registers = thread.current_registers();
    let arguments = current_frame.get_argument_list(argument_list_index);
    let mut stdout = stdout();

    for ((argument_index, _), argument_type) in arguments
        .0
        .iter()
        .zip(function.r#type().value_parameters.iter())
    {
        match argument_type {
            Type::String => {
                let string = current_registers
                    .strings
                    .get(*argument_index as usize)
                    .as_value();

                stdout.write_all(string.as_bytes()).unwrap();
            }
            _ => unreachable!(),
        }
    }

    stdout.write_all(b"\n").unwrap();
}
