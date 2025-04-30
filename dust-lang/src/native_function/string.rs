use crate::{DustString, Instruction, Type, instruction::CallNative, risky_vm::Thread};

pub fn to_string(instruction: Instruction, thread: &mut Thread) {
    let CallNative {
        destination,
        function: _,
        argument_list_index,
    } = CallNative::from(instruction);

    let current_frame = thread.current_frame();
    let current_registers = thread.current_registers();
    let arguments = current_frame.get_argument_list(argument_list_index);

    let string = match arguments.1[0] {
        Type::Integer => {
            let integer = current_registers
                .integers
                .get(arguments.0[0] as usize)
                .as_value();

            DustString::from(integer.to_string())
        }
        _ => unreachable!(),
    };

    *thread
        .current_registers_mut()
        .strings
        .get_mut(destination as usize)
        .as_value_mut() = DustString::from(string);
}
